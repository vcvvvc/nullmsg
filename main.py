import threading
import os
import time
import requests
from concurrent.futures import ThreadPoolExecutor, as_completed

from http.server import HTTPServer, SimpleHTTPRequestHandler
from src.BlockBeats import beat
from src.TreeNews import tree
from src.BWEnews import bwe
from src.jinse import Js
from src.odaily import oda
from src.tuoluo import Tl
from src.PanNews import PAN

def run():
    """
    优化后的主运行函数，使用线程池管理线程
    """
    try:
        # 定义任务列表
        tasks = [
            # ('Jinse', Js.get_news),
            # ('Odaily', oda.get_news),
            # ('Tuoluo', Tl.get_news),
            # ('PanNews', PAN.get_news),
            # ('BWENews', bwe.get_news),
            # ('TreeNews', tree.get_news),
            ('Beats', beat.get_news),
            ('WebServer', server)
        ]
        
        # 使用线程池执行任务
        with ThreadPoolExecutor(max_workers=len(tasks)) as executor:
            # 提交所有任务
            future_to_task = {
                executor.submit(task_func): task_name 
                for task_name, task_func in tasks
            }
            
            # 处理完成的任务
            for future in as_completed(future_to_task):
                task_name = future_to_task[future]
                try:
                    result = future.result()
                    print(f"任务 {task_name} 执行完成")
                except Exception as e:
                    print(f"任务 {task_name} 执行失败: {str(e)}")
                    
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
        time.sleep(60)
        try:
            pushurl = 'https://pushmsg24h.onrender.com'
            sendurl = 'https://bark-test-cje9.onrender.com'
            headers2 = {
                'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15'
            }
            res = requests.get(sendurl, headers=headers2, timeout=60)
            if res.status_code != 200 or res.status_code != 404:
                res = requests.get(sendurl, headers=headers2, timeout=60)

            requests.get(pushurl, headers=headers2, timeout=60)
            if res.status_code != 200 or res.status_code != 404:
                requests.get(pushurl, headers=headers2, timeout=60)
        except:
            continue

if __name__ == '__main__':
    print("=" * 50)
    print("🚀 启动 nullmsg 服务...")
    print("=" * 50)
    print("📋 服务列表:")
    print("   • Jinse 新闻服务")
    print("   • Odaily 新闻服务") 
    print("   • Tuoluo 新闻服务")
    print("   • PanNews 新闻服务")
    print("   • BWEnews 新闻服务")
    print("   • Treenews 新闻服务")
    print("   • BlockBeats 新闻服务")
    print("   • Web 服务器")
    print("   • 心跳检测服务")
    print("=" * 50)
    
    try:
        run()
        print("✅ 所有服务启动成功！")
        print("💓 心跳检测服务已启动...")
        heartbeat()
    except KeyboardInterrupt:
        print("\n⚠️  收到中断信号，正在关闭服务...")
        print("👋 服务已停止")
    except Exception as e:
        print(f"❌ 服务启动失败: {str(e)}")
        print("🔍 请检查日志文件或配置")
