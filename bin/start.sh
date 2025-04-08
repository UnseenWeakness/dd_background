#!/bin/bash

# 项目目录
PROJECT_DIR="./"
# 可执行文件名
EXECUTABLE="dd_background"
# 日志文件
LOG_FILE="$PROJECT_DIR/server.log"
# PID 文件，用于记录进程 ID
PID_FILE="$PROJECT_DIR/server.pid"

# 切换到项目目录
cd "$PROJECT_DIR" || {
    echo "无法切换到项目目录: $PROJECT_DIR"
    exit 1
}

# 编译项目
echo "开始编译项目..."
cargo build --release || {
    echo "编译失败"
    exit 1
}
echo "编译完成"

# 检查是否已有进程在运行
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if ps -p "$OLD_PID" > /dev/null 2>&1; then
        echo "已有进程运行 (PID: $OLD_PID)，请先停止"
        exit 1
    else
        echo "发现旧 PID 文件，但进程已不存在，清理中..."
        rm -f "$PID_FILE"
    fi
fi

# 启动程序，后台运行并记录日志
echo "启动 $EXECUTABLE..."
nohup "$PROJECT_DIR/target/release/$EXECUTABLE" > "$LOG_FILE" 2>&1 &

# 获取新进程的 PID 并保存
NEW_PID=$!
echo "$NEW_PID" > "$PID_FILE"
echo "服务已启动，PID: $NEW_PID，日志输出到: $LOG_FILE"