import requests
import time
import json

from src.push_msg import Pmsg

class Treenews(object):
    def __init__(self):
        self.topid = "1962701984504652098"
        self.date = ''

    def get_news(self):
        while True:
            m_json = ''
            url = 'https://news.treeofalpha.com/api/news?limit=5'
            headers = {
                'User-Agent': 'Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1',
            }
            try:
                res = requests.get(url, headers=headers, timeout=30)
                if res.status_code == 200:
                    tree_json = json.loads(res.text)
                    # print(tree_json[0]['_id'])

                    for news in tree_json:    
                        nid = news['_id']

                        if self.topid == nid:
                            time.sleep(380)
                            break

                        title = news['title']
                        link = news['url'].replace("https://", "www.")       
                        content = ' '


                        if news['source'] == "Twitter":
                            # 使用 ': '作为分隔符，并且设置 maxsplit=1 表示只分割一次
                            parts = title.split(': ', 1)

                            # 分割后会得到一个列表，我们检查一下列表长度确保分割成功
                            if len(parts) == 2:
                                # 列表的第一个元素是 title
                                title = parts[0]
                                # 列表的第二个元素是 content
                                content = parts[1]
                        Pmsg.sendmeg(link, content, title, "Treenews")
                        time.sleep(5)
                        
                    self.topid = tree_json[0]['_id']
                    print("Treenews_发送成功 timesleep")

                time.sleep(600)
            except Exception as e:
                print(e)
                time.sleep(600)

tree = Treenews()