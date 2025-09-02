#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
心跳检测脚本 - 用于GitHub Action
每10分钟访问指定的URL以保持服务活跃
"""

import requests
import time
import sys

def heartbeat():
    """执行心跳检测"""
    pushurl = 'https://pushmsg24h.onrender.com'
    sendurl = 'https://bark-test-cje9.onrender.com'
    headers = {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15'
    }
    
    print('=' * 50)
    print('💓 开始心跳检测...')
    print(f'🕐 访问时间: {time.strftime("%Y-%m-%d %H:%M:%S")}')
    print('=' * 50)
    
    success_count = 0
    
    # 访问第一个URL
    try:
        print(f'🌐 正在访问: {pushurl}')
        res1 = requests.get(pushurl, headers=headers, timeout=30)
        print(f'✅ pushurl 状态码: {res1.status_code}')
        success_count += 1
    except Exception as e:
        print(f'❌ pushurl 访问失败: {str(e)}')
        
    # 访问第二个URL
    try:
        print(f'🌐 正在访问: {sendurl}')
        res2 = requests.get(sendurl, headers=headers, timeout=30)
        print(f'✅ sendurl 状态码: {res2.status_code}')
        success_count += 1
    except Exception as e:
        print(f'❌ sendurl 访问失败: {str(e)}')
    
    print('=' * 50)
    print(f'📊 检测结果: {success_count}/2 个URL访问成功')
    print('💓 心跳检测完成')
    print('=' * 50)
    
    return success_count

if __name__ == '__main__':
    try:
        success_count = heartbeat()
        if success_count > 0:
            print('✅ 心跳检测任务成功完成')
            sys.exit(0)
        else:
            print('⚠️  所有URL访问失败')
            sys.exit(1)
    except Exception as e:
        print(f'❌ 心跳检测任务失败: {str(e)}')
        sys.exit(1)
