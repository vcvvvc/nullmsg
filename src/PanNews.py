import requests
import time
import json

from src.push_msg import Pmsg

class Pannews(object):
    def __init__(self):
        self.topid = 0
        self.art_id = '1'

    def get_news(self):
        while True:
            url = 'https://www.panewslab.com/webapi/flashnews?LId=1&LastTime=0&Rn=20&tw=0'
            headers = {
                'Host': 'www.panewslab.com',
                'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:142.0) Gecko/20100101 Firefox/142.0',
                'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
                'Accept-Language': 'zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2'
            }
            try:
                res = requests.get(url, headers=headers, timeout=30)
                if res.status_code == 200:
                    m_json = json.loads(res.text)
                    m_list = m_json['data']['flashNews']

                    m_flashNews = m_list[0]['list']
                    if self.art_id == m_flashNews[0]['id']:
                        print('pan_art_id == json_art_id')
                        time.sleep(300)
                        continue
                    
                    for news in m_flashNews:
                        try:
                            content = news['desc']
                            title = news['title']
                            news_id = news['id']
                            news_url = 'www.panewslab.com/zh/articles/{0}'.format(news_id)
                            if self.art_id == news_id:
                                break
                            Pmsg.sendmeg(news_url, content.replace("\r\n", "\n"), title, "Pannews")
                            time.sleep(3)
                        except Exception as e:
                            print(e)

                    self.art_id = m_flashNews[0]['id']
                    print("pan_发送成功 timesleep")
                time.sleep(600)
            except Exception as e:
                print(e)
                time.sleep(600)

Pan = Pannews()