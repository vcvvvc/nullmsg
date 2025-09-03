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
from src.PanNews import PAN

def run():
    """
    启动所有任务为后台守护线程并立即返回，让主线程继续执行 heartbeat。
    """
    try:
        tasks = [
            ('Jinse', Js.get_news),
            ('Odaily', oda.get_news),
            ('CoinTime', coin_time.get_news),
            ('Tuoluo', Tl.get_news),
            ('PanNews', PAN.get_news),
            ('BWENews', bwe.get_news),
            ('TreeNews', tree.get_news),
            ('Beats', beat.get_news),
            ('WebServer', server)
        ]

        for task_name, task_func in tasks:
            t = threading.Thread(target=task_func, name=f"{task_name}Thread", daemon=True)
            t.start()
            print(f"✅ 任务 {task_name} 已作为守护线程启动: {t.name}")

        return True
    except Exception as e:
        print(f"运行主函数时发生错误: {str(e)}")
        raise


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
            pushurl = 'https://pushmsg24h.onrender.com'
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
        print("✅ 所有服务启动成功！")
        print("💓 心跳检测服务已启动...")
        heartbeat()
    except Exception as e:
        print(f"❌ 服务启动失败: {str(e)}")
        print("🔍 请检查日志文件或配置")
