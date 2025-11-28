import threading
import os
import time
import requests
from concurrent.futures import ThreadPoolExecutor, as_completed

from http.server import HTTPServer, SimpleHTTPRequestHandler
from src.CoinTime import coin_time
from src.BlockBeats import beat
from src.TreeNews import tree
from src.BWEnews import bwe
from src.jinse import Js
from src.odaily import oda
from src.tuoluo import Tl
from src.PanNews import Pan

# 全局变量，用于存储和管理线程
running_threads = {}
# 线程锁，用于保护对 running_threads 的访问
threads_lock = threading.Lock()


def run():
    """
    启动所有任务为后台守护线程并监控其状态。
    返回值：
        bool: 所有任务是否成功启动
    """
    # 按优先级排序的任务列表
    tasks = [
        # 心跳服务放入线程运行
        ('Heartbeat', heartbeat),
        # 新闻采集服务
        ('Odaily', oda.get_news),
        ('PanNews', Pan.get_news),
        ('BWENews', bwe.get_news),
        ('Beats', beat.get_news),
        # ---
        # ('Jinse', Js.get_news),
        # ('CoinTime', coin_time.get_news),
        # ('Tuoluo', Tl.get_news),
        # ('TreeNews', tree.get_news),
    ]

    try:
        # 清理已存在的线程记录
        with threads_lock:
            running_threads.clear()

        # 启动所有任务
        for task_name, task_func in tasks:
            try:
                # 创建新线程
                thread = threading.Thread(
                    target=task_func,
                    name=f"{task_name}Thread",
                    daemon=True
                )
                # 保存线程引用
                with threads_lock:
                    running_threads[task_name] = {
                        'thread': thread,
                        'function': task_func,
                        'start_time': time.time(),
                        'restarts': 0
                    }
                # 启动线程
                thread.start()
                print(f"✅ 任务 {task_name} 已启动: {thread.name}")

            except Exception as e:
                print(f"❌ 任务 {task_name} 启动失败: {str(e)}")
                continue  # 继续启动下一个

        # 启动线程监控
        monitor_thread = threading.Thread(
            target=monitor_threads,
            name="ThreadMonitor",
            daemon=True
        )
        monitor_thread.start()
        print("✅ 线程监控服务已启动")

        return True

    except Exception as e:
        print(f"❌ 运行主函数时发生错误: {str(e)}")
        raise


def monitor_threads():
    """
    监控线程状态，重启异常退出的线程
    """
    while True:
        try:
            # 使用线程锁复制当前线程字典，避免遍历时的修改
            with threads_lock:
                threads_to_check = dict(running_threads)

            for task_name, info in threads_to_check.items():
                thread = info['thread']
                if not thread.is_alive():
                    print(f"⚠️ 检测到任务 {task_name} 已停止，尝试重启...")
                    try:
                        # 创建新线程
                        new_thread = threading.Thread(
                            target=info['function'],
                            name=f"{task_name}Thread",
                            daemon=True
                        )

                        # 使用线程锁更新线程信息
                        with threads_lock:
                            if task_name in running_threads:  # 再次检查任务是否还存在
                                running_threads[task_name]['thread'] = new_thread
                                running_threads[task_name]['restarts'] += 1
                                running_threads[task_name]['start_time'] = time.time()
                                # 启动新线程
                                new_thread.start()
                                print(f"✅ 任务 {task_name} 已重启 (重启次数: {running_threads[task_name]['restarts']})")
                    except Exception as e:
                        print(f"❌ 任务 {task_name} 重启失败: {str(e)}")

        except Exception as e:
            print(f"❌ 线程监控异常: {str(e)}")

        time.sleep(15)  # 每15秒检查一次


def server():
    webdir = './index'  # 设置网站的根目录为程序所在路径
    port = 80  # 设置一个端口
    # 注意：os.chdir 会改变整个进程的工作目录
    if os.path.exists(webdir):
        os.chdir(webdir)
    else:
        print(f"⚠️ 警告: 目录 {webdir} 不存在，将在当前目录启动 Server")

    server_address = ('', port)  # 设置服务器地址
    server_obj = HTTPServer(server_address, SimpleHTTPRequestHandler)  # 创建服务器对象
    print(f"🚀 WebServer 正在监听端口 {port}...")
    server_obj.serve_forever()  # 启动服务器


def heartbeat():  # render无活动时间久了会暂停服务，定时get活动一下
    while True:
        time.sleep(10)
        try:
            pushurl = 'https://pushnews.onrender.com'
            sendurl = 'https://bark-test-cje9.onrender.com'
            headers2 = {
                'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15'
            }
            # 使用简单的异常处理，避免因为网络抖动导致线程崩溃
            try:
                requests.get(sendurl, headers=headers2, timeout=10)
                requests.get(pushurl, headers=headers2, timeout=10)
            except requests.RequestException:
                pass  # 忽略暂时的网络错误

        except Exception as e:
            print(f"❌ 心跳服务异常: {e}")
            time.sleep(5)  # 发生错误稍微停顿
            continue


if __name__ == '__main__':

    try:
        # 1. 启动后台线程（心跳 + 爬虫）
        run()
        print("✅ 所有后台采集及心跳服务已启动")

        # 2. 在主进程启动 WebServer (这将阻塞主进程，保持程序运行)
        server()

    except Exception as e:
        print(f"❌ 服务启动失败: {str(e)}")
        print("🔍 请检查日志文件或配置")