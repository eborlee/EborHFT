#!/bin/bash
set -e

PYTHON_BIN=${1:-python3.8}

if ! command -v $PYTHON_BIN &> /dev/null; then
    echo "❌ 找不到 Python：$PYTHON_BIN，请确认已安装"
    exit 1
fi

if [ -d .venv ]; then
    echo "✅ .venv 已存在，跳过创建，可手动删除后重建"
else
    echo "🔧 创建虚拟环境..."
    $PYTHON_BIN -m venv .venv
fi

source .venv/bin/activate

echo "⬆️ 安装依赖..."
pip install -U pip
pip install -r requirements.txt

echo "✅ Python 虚拟环境配置完成"
