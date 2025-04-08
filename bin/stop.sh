#!/bin/bash

# 项目目录
PROJECT_DIR="./"
# PID 文件
PID_FILE="$PROJECT_DIR/server.pid"

# 检查 PID 文件是否存在
if [ ! -f "$PID_FILE" ]; then
    echo "未找到 PID 文件，服务可能未运行"
    exit 1
fi

# 获取当前 PID
PID=$(cat "$PID_FILE")
if ps -p "$PID" > /dev/null 2>&1; then
    echo "停止进程 (PID: $PID)..."
    kill -TERM "$PID"
    # 等待进程结束
    while ps -p "$PID" > /dev/null 2>&1; do
        sleep 1
    done
    echo "进程已停止"
    rm -f "$PID_FILE"
else
    echo "PID 文件存在，但进程未运行，清理 PID 文件..."
    rm -f "$PID_FILE"
fi