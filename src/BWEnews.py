import json
import time
import requests
import feedparser

from src.push_msg import Pmsg

class BWEnews(object):
    def __init__(self):
        self.topid = None
        self.date = ''

    def get_entry_id(self, entry):
        return entry.get('id', entry.get('link'))

    def get_news(self):
        while True:
            m_json = ''
            url = 'https://rss-public.bwe-ws.com/'
            headers = {
                'User-Agent': 'Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1',
            }
            try:
                res = requests.get(url, headers=headers, timeout=30)
                if res.status_code == 200:
                    feed = feedparser.parse(res.content)

                    if not feed.entries:
                        print("Feed 为空或解析失败，稍后重试。")
                        time.sleep(300)
                        continue

                    current_latest_entry = feed.entries[0]
                    current_latest_id = self.get_entry_id(current_latest_entry)

                    if current_latest_id == self.topid:
                        print('od_topid == json_topid')
                        time.sleep(300)
                        continue

                    # 从最新的一条开始遍历，直到找到上次记录的ID
                    for entry in feed.entries:
                        entry_id = self.get_entry_id(entry)
                        # print("entry: ", entry, "\n")
                         # 1. 直接使用 '<br/>' 作为分隔符对原始字符串进行分割
                        title_part, separator, content_part = entry.title.partition('<br/>')
                    
                        # 2. 对分割后的 content_part 进行 <br/> -> \n 的替换
                        cleaned_content = content_part.replace('<br/>', '\n').replace('<br>', '\n').replace('—', ' ')
                        
                        # 3. 清理首尾空格，得到最终结果
                        final_title = title_part.strip()
                        final_content = cleaned_content.strip()
                        link = entry.link.replace('https://', 'www.')
                        if entry_id == self.topid:
                            break                        
                        else:
                            # print("=" * 50)
                            # print(f"【标题】: {final_title}")
                            # print(f"【内容】:\n{final_content}\n")
                            # print(f"【链接】: {entry.link}")
                            # print("-" * 50)
                            Pmsg.sendmeg(link, final_content, final_title, "BWEnews")

                    self.topid = current_latest_id
                    print("bwe_发送成功 timesleep")
                time.sleep(600)
            except Exception as e:
                print(e)
                time.sleep(600)


bwe = BWEnews()