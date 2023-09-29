import requests
import time
import json

from src.push_msg import Pmsg

class DODO(object):
    def __init__(self):
        self.topid = '0'
        self.date = ''

    def get_group(self):
        url = 'https://www.imdodo.com/s/233780'
        headers = {
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7',
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15',
            'Host': 'www.imdodo.com'
        }
        content = '准备注册准备注册准备注册'

        while True:
            try:
                res = requests.get(url, headers=headers, timeout=30)
                if res.status_code == 200:
                    if res.headers['Content-Type'] == 'text/html;charset=UTF-8':
                        for _ in range(10):
                            Pmsg.sendmeg(url, content, "{0}, 已被注册".format(url))
                            time.sleep(5)
                        url = 'https://www.imdodo.com/s/233785'
                        content = "785已经注册，准备注册788，1000888"
                        time.sleep(100)
                        continue

                    m_json = json.loads(res.text)
                    if m_json['status'] == -500:
                        time.sleep(600)
                        continue

            except Exception as e:
                time.sleep(600)
                print(e)


    def get_news(self):
        return
        while True:
            url = 'https://api.jinse.cn/noah/v2/lives?limit=20&reading=false&source=web'
            headers = {
                'Origin': 'https://www.jinse.com',
                'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15',
                'Referer': 'https://www.jinse.com/',
                'Host': 'api.jinse.cn'
            }
            try:
                res = requests.get(url, headers=headers, timeout=30)
                if res.status_code == 200:
                    m_json = json.loads(res.text)
                    if self.topid == m_json['top_id']:
                        print('jin_topid == json_topid' )
                        time.sleep(600)
                        continue

                    m_list = m_json['list']
                    m_lives = m_list[0]['lives']
                    for n in range(len(m_lives)):
                        try:
                            content = m_lives[n]['content']
                            content_prefix = m_lives[n]['content_prefix']
                            id = m_lives[n]['id']
                            news_url = 'www.jinse.cn/lives/{0}.html'.format(id)
                            if self.topid == id:
                                print("jin_topid == json_id")
                                break
                            else:
                                Pmsg.sendmeg(news_url, content, content_prefix)
                                time.sleep(3)
                        except Exception as e:
                            print(e)

                    if self.date != m_list[0]['date'] and self.date != '':
                        for n in range(len(m_lives)):
                            try:
                                m_lives = m_list[1]['lives']
                                content = m_lives[n]['content']
                                content_prefix = m_lives[n]['content_prefix']
                                id = m_lives[n]['id']
                                news_url = 'https://www.jinse.cn/lives/{0}.html'.format(id)
                                if self.topid == id:
                                    print("jin_topid == json_id")
                                    break
                                else:
                                    Pmsg.sendmeg(news_url, content, content_prefix)
                                    time.sleep(3)
                            except Exception as e:
                                print(e)
                    self.date = m_list[0]['date']
                    self.topid = m_json['top_id']
                    print("jin_发送成功 timesleep")
                time.sleep(600)
            except Exception as e:
                time.sleep(600)
                print(e)

dodo = DODO()