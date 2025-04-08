#!/bin/bash

# 项目目录
PROJECT_DIR="./"
# 可执行文件名
EXECUTABLE="dd_background"
# PID 文件
PID_FILE="$PROJECT_DIR/server.pid"
# 日志文件
LOG_FILE="$PROJECT_DIR/server.log"

# 检查 PID 文件是否存在
if [ ! -f "$PID_FILE" ]; then
    echo "未找到 PID 文件，可能是服务未运行，直接启动..."
    "$PROJECT_DIR/start.sh"
    exit $?
fi

# 获取当前 PID
PID=$(cat "$PID_FILE")
if ps -p "$PID" > /dev/null 2>&1; then
    echo "停止当前进程 (PID: $PID)..."
    kill -TERM "$PID"
    # 等待进程结束
    while ps -p "$PID" > /dev/null 2>&1; do
        sleep 1
    done
    echo "进程已停止"
else
    echo "PID 文件存在，但进程未运行，清理 PID 文件..."
    rm -f "$PID_FILE"
fi

# 调用启动脚本
echo "重新启动服务..."
"$PROJECT_DIR/start.sh"