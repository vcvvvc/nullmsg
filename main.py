import threading
import os
import time

from http.server import HTTPServer, CGIHTTPRequestHandler
from src.jinse import Js
from src.odaily import oda
from src.tuoluo import Tl

def run():
    t1 = threading.Thread(target=Js.get_news, args=())
    t2 = threading.Thread(target=oda.get_news, args=())
    t3 = threading.Thread(target=Tl.get_news, args=())
    # t4 = threading.Thread(target=coin_time.get_news, args=())
    t1.start()
    time.sleep(3)
    t2.start()
    time.sleep(3)
    t3.start()
    # t4.start()


if __name__ == '__main__':
    webdir = './index'  # 设置网站的根目录为程序所在路径
    port = 80  # 设置一个端口
    os.chdir(webdir)
    server_address = ('', port)  # 设置服务器地址
    server_obj = HTTPServer(server_address, CGIHTTPRequestHandler)  # 创建服务器对象
    # run()
    Js.get_news()
    # oda.get_news()
    # ti.get_news()
    # Tl.get_news()
    # coin_time.get_news()
    server_obj.serve_forever()  # 启动服务器
