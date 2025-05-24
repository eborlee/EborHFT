#!/bin/bash
set -e

# 进入 docker 文件夹所在目录
cd "$(dirname "$0")"

# 提取版本号（从项目根目录的 Cargo.toml）
VERSION=$(grep '^version' ../Cargo.toml | head -n 1 | cut -d '"' -f2)

# 镜像名
IMAGE_NAME="trade-monitor:${VERSION}"

echo "🔨 Building Docker image: ${IMAGE_NAME}"
docker build -f Dockerfile -t "${IMAGE_NAME}" ..
