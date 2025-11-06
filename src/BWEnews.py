import json
import time
import requests
import feedparser

from src.push_msg import Pmsg

class BWEnews(object):
    def __init__(self):
        self.topid = None
        self.date = ''

        self.last_request_time = 0  # 记录上次请求时间
        self.min_interval = 60  # 最小请求间隔60秒
        self.adaptive_interval = 120  # 自适应间隔，初始120秒

    def get_entry_id(self, entry):
        return entry.get('id', entry.get('link'))

    def get_news(self):
        while True:
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

                    # 获取最新的消息（feed.entries[-1]是最新的，feed.entries[0]是最旧的）
                    current_latest_entry = feed.entries[-1]
                    current_latest_id = self.get_entry_id(current_latest_entry)

                    # 如果最新消息的ID就是上次记录的ID，说明没有新消息，直接跳过
                    if current_latest_id == self.topid:
                        print('bwe_topid == json_topid')
                        time.sleep(59)
                        continue

                    # 从最新的一条开始遍历，直到找到上次记录的ID
                    processed_count = 0
                    # 从最新消息开始向前处理（倒序遍历）
                    for i in range(len(feed.entries) - 1, -1, -1):
                        entry = feed.entries[i]
                        entry_id = self.get_entry_id(entry)
                        
                        # 1. 直接使用 '<br/>' 作为分隔符对原始字符串进行分割
                        title_part, separator, content_part = entry.title.partition('<br/>')
                    
                        # 2. 对分割后的 content_part 进行 <br/> -> \n 的替换
                        cleaned_content = content_part.replace('<br/>', '\n').replace('<br>', '\n').replace('—', ' ')
                        
                        # 3. 清理首尾空格，得到最终结果
                        final_title = title_part.strip()
                        final_content = cleaned_content.strip()
                        link = entry.link.replace('https://', '')
                        
                        # 如果遇到上次记录的ID，说明之前的消息都已经处理过了，停止处理
                        if entry_id == self.topid:
                            break
                        else:
                            # 发送消息
                            Pmsg.sendmeg(link, final_content, final_title, "BWEnews")
                            processed_count += 1
                            print(f"已处理消息 ID: {entry_id}")

                    # 处理完所有新消息后，更新topid为当前最新条目的ID
                    self.topid = current_latest_id

                    if processed_count > 0:
                        print(f"bwe_发送成功，共处理 {processed_count} 条消息")
                    else:
                        print("bwe_没有新消息")
                        
                time.sleep(60)
            except Exception as e:
                print(f"处理过程中出现异常: {e}")
                time.sleep(300)


bwe = BWEnews()
