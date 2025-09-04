import requests
import time
import json
import re

from src.push_msg import Pmsg

class BlockBeats(object):
    def __init__(self):
        self.topid = 0
        self.art_id = 'en5mtun6'

    def get_news(self):
        while True:
            url = 'https://api.blockbeats.cn/v2/newsflash/list?page=1&limit=10'
            headers = {
                'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36',
                'Host': 'api.blockbeats.cn'
            }
            try:
                res = requests.get(url, headers=headers, timeout=30)
                if res.status_code == 200:
                    m_json = json.loads(res.text)
                    m_list = m_json['data']['list']
                    
                    # 检查列表是否为空
                    if not m_list:
                        print("BlockBeats: 新闻列表为空")
                        time.sleep(300)
                        continue
                    
                    m_id = m_list[0]['article_id']
                    
                    # https://www.theblockbeats.info/flash/310574
                    for bnews in m_list:
                        try:                            
                            article_id = bnews.get('article_id')
                            if self.topid == article_id:
                                print('beats_topid == json_topid' )
                                time.sleep(300)
                                break
                                
                            title = bnews.get('title', '')
                            content_html = bnews.get('content', '')
                            content_text = re.sub(r'<[^>]+>', '', content_html)
                            link = f"www.theblockbeats.info/flash/{article_id}"
                            # print(title, link)
                            # print(content_text)

                            Pmsg.sendmeg(link, content_text, title, "BlockBeats")
                            time.sleep(3)
                        except Exception as e:
                            print(e)

                    self.topid = m_id
                    print("beats_发送成功 timesleep")
                time.sleep(300)
            except Exception as e:
                print(e)
                time.sleep(600)

beat = BlockBeats()