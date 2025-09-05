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
        # 核心服务优先启动
        ('WebServer', server),
        # 新闻采集服务
        # ('Jinse', Js.get_news),
        ('Odaily', oda.get_news),
        # ('CoinTime', coin_time.get_news),
        # ('Tuoluo', Tl.get_news),
        ('PanNews', Pan.get_news),
        ('BWENews', bwe.get_news),
        ('TreeNews', tree.get_news),
        ('Beats', beat.get_news),
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
                
                # 对于核心服务，等待其完全启动
                if task_name == 'WebServer':
                    time.sleep(1)  # 给予WebServer启动时间
                    if not thread.is_alive():
                        raise Exception(f"{task_name} 启动失败")
                
            except Exception as e:
                print(f"❌ 任务 {task_name} 启动失败: {str(e)}")
                # 如果是核心服务启动失败，则终止整个程序
                if task_name == 'WebServer':
                    raise
                continue  # 其他服务失败则继续启动下一个

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
        
        time.sleep(15)  # 每5秒检查一次


def server():
    webdir = './index'  # 设置网站的根目录为程序所在路径
    port = 80  # 设置一个端口
    os.chdir(webdir)
    server_address = ('', port)  # 设置服务器地址
    server_obj = HTTPServer(server_address, SimpleHTTPRequestHandler)  # 创建服务器对象
    server_obj.serve_forever()  # 启动服务器

def heartbeat(): #render无活动时间久了会暂停服务，定时get活动一下
    while True:
        time.sleep(10)
        try:
            pushurl = 'https://pushnews.onrender.com'
            sendurl = 'https://bark-test-cje9.onrender.com'
            headers2 = {
                'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15'
            }
            res = requests.get(sendurl, headers=headers2, timeout=10)
            if res.status_code != 200 and res.status_code != 404:
                res = requests.get(sendurl, headers=headers2, timeout=20)

            requests.get(pushurl, headers=headers2, timeout=10)
            if res.status_code != 200 and res.status_code != 404:
                requests.get(pushurl, headers=headers2, timeout=20)

            time.sleep(20)    
        except Exception as e:
            print(e)
            continue

if __name__ == '__main__':
   
    try:
        run()
        print("✅ 所有采集服务启动")
        print("💓 心跳检测服务启动...")
        heartbeat()
    except Exception as e:
        print(f"❌ 服务启动失败: {str(e)}")
        print("🔍 请检查日志文件或配置")
