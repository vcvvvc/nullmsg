#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
测试TreeNews WebSocket连接
"""

import asyncio
import websockets
import json

async def test_websocket():
    """测试WebSocket连接"""
    url = 'wss://www.madnews.io/ws'
    
    try:
        print(f"🔗 正在连接 {url}...")
        async with websockets.connect(url) as websocket:
            print("✅ WebSocket连接成功！")
            
            # 等待接收消息
            print("📡 等待接收消息...")
            try:
                async for message in websocket:
                    print(f"📨 收到消息: {message}")
                    try:
                        data = json.loads(message)
                        print(f"📊 解析后的数据: {json.dumps(data, indent=2, ensure_ascii=False)}")
                    except json.JSONDecodeError:
                        print(f"❌ JSON解析失败: {message}")
            except websockets.exceptions.ConnectionClosed:
                print("⚠️ 连接已关闭")
                
    except Exception as e:
        print(f"❌ 连接失败: {e}")

if __name__ == "__main__":
    print("🚀 开始测试TreeNews WebSocket连接...")
    try:
        asyncio.run(test_websocket())
    except KeyboardInterrupt:
        print("\n⚠️ 测试被中断")
    except Exception as e:
        print(f"❌ 测试失败: {e}")
