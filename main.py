import threading
import os
import time
import requests

from http.server import HTTPServer, CGIHTTPRequestHandler
from src.jinse import Js
from src.odaily import oda
from src.tuoluo import Tl

def run():
    t1 = threading.Thread(target=Js.get_news, args=())
    t2 = threading.Thread(target=oda.get_news, args=())
    t3 = threading.Thread(target=Tl.get_news, args=())
    t4 = threading.Thread(target=server, args=())
    t1.start()
    time.sleep(3)
    t2.start()
    time.sleep(3)
    t3.start()
    t4.start()

def server():
    webdir = './index'  # 设置网站的根目录为程序所在路径
    port = 80  # 设置一个端口
    os.chdir(webdir)
    server_address = ('', port)  # 设置服务器地址
    server_obj = HTTPServer(server_address, CGIHTTPRequestHandler)  # 创建服务器对象
    server_obj.serve_forever()  # 启动服务器

def heartbeat(): #render无活动时间久了会暂停服务，定时get活动一下
    while True:
        time.sleep(40)
        pushurl = 'https://pushmsg.onrender.com'
        sendurl = 'https://bark-test-cje9.onrender.com'
        headers2 = {
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15'
        }
        res = requests.get(sendurl, headers=headers2, timeout=30)
        if res.status_code != 200:
            res = requests.get(sendurl, headers=headers2, timeout=30)

        requests.get(pushurl, headers=headers2, timeout=30)
        if res.status_code != 200:
            requests.get(pushurl, headers=headers2, timeout=30)

if __name__ == '__main__':
    run()
    heartbeat()
    # Js.get_news()
    # oda.get_news()
    # ti.get_news()
    # Tl.get_news()
    # coin_time.get_news()
