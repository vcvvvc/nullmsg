import threading
import os
import time
import requests
import socket  # 引入 socket 用于设置全局超时
from functools import partial
from http.server import HTTPServer, SimpleHTTPRequestHandler

# 引入你的业务模块
from src.CoinTime import coin_time
from src.BlockBeats import beat
from src.TreeNews import tree
from src.BWEnews import bwe
from src.jinse import Js
from src.odaily import oda
from src.tuoluo import Tl
from src.PanNews import Pan

# ==========================================
# 核心配置区域
# ==========================================

# 1. 设置全局 Socket 超时 (秒)
# 这是解决"运行久了卡顿"的关键。如果爬虫请求超过 30秒 没反应，
# 就会强制抛出异常，让监控线程捕获并重启它。
socket.setdefaulttimeout(30)

# 全局变量
running_threads = {}
threads_lock = threading.Lock()


# ==========================================
# 线程管理逻辑
# ==========================================

def thread_wrapper(func, name):
    """
    线程包装器：
    捕获所有未处理的异常，确保线程死亡时能留下日志，
    而不是静默消失。
    """
    print(f"▶️ [启动] 任务 {name} 开始运行...")
    try:
        func()
    except Exception as e:
        # 这里捕获到的通常是 socket.timeout 或解析错误
        print(f"❌ [崩溃] 任务 {name} 异常停止: {str(e)}")
    finally:
        print(f"⏹️ [结束] 任务 {name} 已退出")


def start_thread_task(task_name, task_func):
    """
    统一的启动/重启线程函数
    """
    try:
        # 创建线程，注意：target 指向 wrapper，args 传递真正的函数
        thread = threading.Thread(
            target=thread_wrapper,
            args=(task_func, task_name),
            name=f"{task_name}Thread",
            daemon=True
        )

        with threads_lock:
            # 如果是重启，增加计数；如果是首次，初始化
            if task_name in running_threads:
                running_threads[task_name]['thread'] = thread
                running_threads[task_name]['restarts'] += 1
                running_threads[task_name]['start_time'] = time.time()
                count = running_threads[task_name]['restarts']
                print(f"🔄 [重启] 正在第 {count} 次重启任务 {task_name}...")
            else:
                running_threads[task_name] = {
                    'thread': thread,
                    'function': task_func,
                    'start_time': time.time(),
                    'restarts': 0
                }

        thread.start()
        return True
    except Exception as e:
        print(f"❌ [错误] 无法启动任务 {task_name}: {e}")
        return False


def monitor_threads():
    """
    守护线程：每隔 15 秒检查一次所有任务是否还活着
    """
    print("🛡️ 线程监控服务已就绪")
    while True:
        try:
            # 1. 获取当前需要检查的任务列表 (复制 keys 以防遍历时字典变化)
            with threads_lock:
                tasks_to_check = list(running_threads.keys())

            # 2. 遍历检查
            for task_name in tasks_to_check:
                # 再次加锁获取最新状态
                with threads_lock:
                    task_info = running_threads.get(task_name)

                # 如果任务存在 且 线程已经不再运行 (is_alive() == False)
                if task_info and not task_info['thread'].is_alive():
                    print(f"⚠️ [监控] 检测到任务 {task_name} 已停止，准备重启...")
                    # 重新启动该任务
                    start_thread_task(task_name, task_info['function'])

        except Exception as e:
            print(f"❌ [监控异常] Monitor 发生错误: {str(e)}")

        time.sleep(15)


# ==========================================
# 业务逻辑与服务
# ==========================================

def run():
    """
    初始化所有后台任务
    """
    # 任务清单
    tasks = [
        ('Heartbeat', heartbeat),  # 心跳保活
        ('Odaily', oda.get_news),  # 新闻源 1
        ('PanNews', Pan.get_news),  # 新闻源 2
        ('BWENews', bwe.get_news),  # 新闻源 3
        ('Beats', beat.get_news),  # 新闻源 4
        # 根据需要取消注释:
        # ('Jinse', Js.get_news),
        # ('CoinTime', coin_time.get_news),
        # ('Tuoluo', Tl.get_news),
        # ('TreeNews', tree.get_news),
    ]

    print("🚀 正在初始化后台任务...")
    with threads_lock:
        running_threads.clear()

    # 批量启动
    for name, func in tasks:
        start_thread_task(name, func)

    # 启动监控线程 (Monitor 自己也是一个守护线程)
    monitor = threading.Thread(target=monitor_threads, name="MonitorThread", daemon=True)
    monitor.start()


def heartbeat():
    """
    心跳服务：防止平台因无流量休眠
    """
    while True:
        time.sleep(10)
        try:
            pushurl = 'https://pushnews.onrender.com'
            sendurl = 'https://bark-test-cje9.onrender.com'
            # 模拟真实浏览器 Header
            headers = {
                'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36'
            }
            # 显式设置 timeout，防止心跳线程自己卡死
            requests.get(sendurl, headers=headers, timeout=20)
            requests.get(pushurl, headers=headers, timeout=20)
        except Exception:
            # 心跳失败忽略即可，保持日志清爽
            pass


def server():
    """
    启动 Web 服务器
    """
    webdir = './index'  # 网站根目录
    port = 80  # 监听端口

    # 检查目录
    if not os.path.exists(webdir):
        print(f"⚠️ 警告: 目录 {webdir} 不存在，将在当前目录启动 Server")
        webdir = '.'

    # 关键修改：使用 functools.partial 固定 directory 参数
    # 这样不需要使用 os.chdir()，避免破坏其他线程的文件路径依赖
    handler_class = partial(SimpleHTTPRequestHandler, directory=webdir)

    server_address = ('', port)

    try:
        httpd = HTTPServer(server_address, handler_class)
        print(f"🌐 WebServer 正在监听端口 {port} (根目录: {webdir})...")
        httpd.serve_forever()
    except Exception as e:
        print(f"❌ WebServer 启动失败: {e}")


# ==========================================
# 主程序入口
# ==========================================

if __name__ == '__main__':
    try:
        # 1. 启动所有后台采集线程 + 监控
        run()
        print("✅ 后台服务已全部启动")

        # 2. 启动 WebServer (这将阻塞主进程，保持程序运行)
        server()

    except KeyboardInterrupt:
        print("\n🛑 程序被手动停止")
    except Exception as e:
        print(f"❌ 主程序发生致命错误: {str(e)}")