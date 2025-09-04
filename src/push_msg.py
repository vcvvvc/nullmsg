import platform
import requests
import threading
import base64
import json
import time

from Crypto.Cipher import AES
from src.config import config

mutex = threading.Lock() # 创建一个锁对象

class PushMsg(object):
    def __init__(self):
        self.serv_host = config.get_server_host()
        self.serv_key = config.get_server_key()
        self.aes_key = config.get_aes_key()
        self.aes_iv = config.get_aes_iv()

        self.headers = {
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15',
        }


    def deal_msg(self, news_url, news_content, news_title, news_platform):
        mutex.acquire()  # 获取锁

        try:
            self.cipher = AES.new(self.aes_key.encode(), AES.MODE_CBC, self.aes_iv.encode())

            data_dict = {
                "title": news_title,
                "body": news_content,
                "url": news_url,
                "sound": "healthnotification",
                "group": news_platform
            }

            m_json = json.dumps(data_dict, ensure_ascii=False)

            # print(m_json)

            # 把字符串转换为字节
            message = m_json.encode()
            # 对字节进行填充，使其长度为16的倍数
            pad_length = 16 - len(message) % 16
            message += bytes([pad_length]) * pad_length
            # 加密字节
            token = self.cipher.encrypt(message)
            # 把加密后的字节转换为base64字符串
            token = base64.b64encode(token).decode()

            return token
        except Exception as e:
            print(e)
        finally:
            mutex.release()




    def sendmeg(self, news_url, news_content, news_title, news_platform: str = " "): #rebder
        ciphertext = self.deal_msg(news_url, news_content, news_title, news_platform)
        time.sleep(1)
        data = {
            "ciphertext": ciphertext,
            # "iv": "{0}".format(self.aes_iv),
        }

        sendurl = 'https://bark-test-cje9.onrender.com/quicknews'

        max_retries = 3
        for attempt in range(max_retries):
            try:
                res = requests.post(sendurl, headers=self.headers, data=data, timeout=30)
                
                # 1. 检查HTTP错误状态码 (4xx 或 5xx)，如果出错会自动抛出异常
                res.raise_for_status() 

                # 如果代码能走到这里，说明 status_code 一定是 2xx (成功)
                print(f'发送成功 ({attempt + 1}/{max_retries} 次): {res.text} ---- {news_platform}')
                
                break             
            except requests.exceptions.RequestException as e:
                print(f'发送失败 (尝试 {attempt + 1}/{max_retries} 次): {e}')
                
                if attempt < max_retries - 1:
                    time.sleep(1) # 等待1秒
                
        # 4. for循环的else子句: 只有当循环正常结束(即没有被break)，才会执行
        else:
            # 如果循环完了都没有break，说明所有重试都失败了
            print(f'错误：在尝试 {max_retries} 次后，消息发送最终失败。')

Pmsg = PushMsg()

