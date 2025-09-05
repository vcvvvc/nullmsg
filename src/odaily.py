import requests
import time
import json
import re

from src.push_msg import Pmsg

class oDaily(object):
    def __init__(self):
        self.topid = '0'
        self.date = ''

    def get_news(self):
        while True:
            m_json = ''
            url = 'https://h5-api.odaily.news/newsflash/page?isImport=false&page=1&size=20'
            headers = {
                'User-Agent': 'Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1',
                'Referer': 'https://m.odaily.news',
            }
            try:
                res = requests.get(url, headers=headers, timeout=30)
                if res.status_code == 200:
                    od_json = json.loads(res.text)
                    m_json = od_json['data']['list']
                    if self.topid == m_json[0]['id']:
                        print('od_topid == json_topid' )
                        time.sleep(300)
                        continue

                    for n in range(len(m_json)):
                        content = m_json[n]['description']
                        content_text = re.sub(r'<[^>]+>', '', content)

                        content_title = m_json[n]['title']
                        news_id = m_json[n]['id']
                        news_url = 'www.odaily.news/newsflash/{0}'.format(news_id)
                        if self.topid == news_id:
                            print("od_topid == json_id")
                            break
                        else:
                            Pmsg.sendmeg(news_url, content_text, content_title, "oDaily")
                            time.sleep(3)

                    self.topid = m_json[0]['id']
                    print("od_发送成功 timesleep")
                time.sleep(600)
            except Exception as e:
                print(e)
                time.sleep(600)

oda = oDaily()