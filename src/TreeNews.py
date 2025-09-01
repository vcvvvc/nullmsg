import requests
import time
import json

from src.push_msg import Pmsg

class Treenews(object):
    def __init__(self):
        self.topid = None
        self.date = ''

    def get_news(self):
        while True:
            m_json = ''
            url = 'https://news.treeofalpha.com/api/news?limit=10'
            headers = {
                'User-Agent': 'Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1',
                'Referer': 'https://m.odaily.news',
            }
            try:
                res = requests.get(url, headers=headers, timeout=30)
                if res.status_code == 200:
                    tree_json = json.loads(res.text)
                    # print(tree_json)
                    for news in tree_json:    
                        id = news['_id']

                        if self.topid == id:
                            time.sleep(600)
                            continue

                        title = news['title']
                        link = news['url'].replace("https://", "www.")       
                        content = ' '                 
                        # print(news['title'])
                        # print(f"🔗 【链接】: {link}")
                        # print(f"📡 【来源】: {news.get('source', 'Unknown')}")
                        # print(f"🆔 【消息ID】: {id}")

                        if news['source'] == "Twitter":
                            # 使用 ': '作为分隔符，并且设置 maxsplit=1 表示只分割一次
                            parts = title.split(': ', 1)

                            # 分割后会得到一个列表，我们检查一下列表长度确保分割成功
                            if len(parts) == 2:
                                # 列表的第一个元素是 title
                                title = parts[0]
                                # 列表的第二个元素是 content
                                content = parts[1]
                        Pmsg.sendmeg(link, content, title)
                        time.sleep(3)
                        
                    
                time.sleep(600)    
            except Exception as e:
                print(e)
                time.sleep(600)

tree = Treenews()