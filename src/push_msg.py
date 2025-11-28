import requests
import base64
import json
import time
from Crypto.Cipher import AES
from src.config import config


class PushMsg(object):
    def __init__(self):
        # 优先读取配置，如果没有配置则给一个默认值或报错
        host = config.get_server_host()
        if not host:
            # 也可以选择在这里硬编码作为兜底，或者抛出异常
            host = 'https://bark-test-cje9.onrender.com'

            # 自动拼接路径，避免硬编码整个 URL
        self.sendurl = f"{host.rstrip('/')}/quicknews"

        self.aes_key = config.get_aes_key()
        self.aes_iv = config.get_aes_iv()

        self.headers = {
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15',
        }

    def deal_msg(self, news_url, news_content, news_title, news_platform):
        """
        加密处理逻辑
        不需要锁(mutex)，因为这里只使用局部变量，天然线程安全。
        """
        try:
            # 1. 每次加密创建一个新的 cipher 对象 (局部变量，不要用 self.cipher)
            # encode() 默认是 utf-8
            cipher = AES.new(self.aes_key.encode('utf-8'), AES.MODE_CBC, self.aes_iv.encode('utf-8'))

            data_dict = {
                "title": news_title,
                "body": news_content,
                "url": news_url,
                "sound": "healthnotification",
                "group": news_platform
            }

            m_json = json.dumps(data_dict, ensure_ascii=False)

            # 2. PKCS7 Padding
            message_bytes = m_json.encode('utf-8')
            pad_length = 16 - len(message_bytes) % 16
            message_bytes += bytes([pad_length]) * pad_length

            # 3. 加密
            encrypted_bytes = cipher.encrypt(message_bytes)

            # 4. Base64 编码
            token = base64.b64encode(encrypted_bytes).decode('utf-8')

            return token

        except Exception as e:
            print(f"❌ 加密失败: {e}")
            return None

    def sendmeg(self, news_url, news_content, news_title, news_platform: str = "Common"):
        # 1. 获取密文
        ciphertext = self.deal_msg(news_url, news_content, news_title, news_platform)

        # 2. 如果加密失败（返回None），直接终止，不要发送空请求
        if not ciphertext:
            print(f"⚠️ 跳过发送: 加密失败 - {news_title}")
            return

        # 移除不必要的 time.sleep(1)

        data = {
            "ciphertext": ciphertext,
        }

        max_retries = 3
        for attempt in range(max_retries):
            try:
                # 使用 self.sendurl 而不是硬编码
                res = requests.post(self.sendurl, headers=self.headers, data=data, timeout=30)
                res.raise_for_status()  # 检查 404, 500 等错误

                print(f'✅ 发送成功: {news_platform} - {news_title[:10]}...')
                break  # 成功则跳出循环

            except requests.exceptions.RequestException as e:
                print(f'⚠️ 发送失败 ({attempt + 1}/{max_retries}): {e}')
                if attempt < max_retries - 1:
                    time.sleep(2)  # 失败后稍微等待再重试
        else:
            print(f'❌ 错误: {news_title} 发送最终失败。')


Pmsg = PushMsg()