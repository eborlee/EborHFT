#!/bin/bash
set -e

# 获取脚本所在目录
cd "$(dirname "$0")"

# 提取版本号
VERSION=$(grep '^version' ../Cargo.toml | head -n 1 | cut -d '"' -f2)

# 统一镜像名称和容器名称
IMAGE_NAME="trade-monitor:${VERSION}"
CONTAINER_NAME="trade-monitor"

# 项目根目录路径（用于挂载）
PROJECT_ROOT=$(cd .. && pwd)

# ✅ 检查 config.toml 和 backup.json 是否为有效文件
REQUIRED_FILES=("config.toml" "backup.json")
for file in "${REQUIRED_FILES[@]}"; do
    path="${PROJECT_ROOT}/${file}"
    if [ -d "$path" ]; then
        echo "❌ 错误：'$path' 是目录，必须是文件。请删除后重新提供。"
        exit 1
    fi
    if [ ! -f "$path" ]; then
        echo "❌ 缺失必要文件：$path"
        echo "👉 请手动提供 ${file} 后再运行此脚本。"
        exit 1
    fi
done

# 生成唯一日志文件名（包含版本 + 启动时间）
TIMESTAMP=$(date "+%Y%m%d-%H%M%S")
LOG_DIR="${PROJECT_ROOT}/logs"
mkdir -p "${LOG_DIR}"                        # 确保 logs 目录存在
LOG_FILE="${LOG_DIR}/run-${VERSION}-${TIMESTAMP}.log"

echo "🚀 启动容器：${IMAGE_NAME}"
echo "📁 日志输出到：${LOG_FILE}"

# 启动容器（前台），重定向 stdout 到日志文件中以获取 container_id
docker run -d \
  --name "${CONTAINER_NAME}" \
  -v "${PROJECT_ROOT}/config.toml:/app/config.toml" \
  -v "${PROJECT_ROOT}/data:/app/data" \
  -v "${PROJECT_ROOT}/scripts:/app/scripts" \
  -v "${PROJECT_ROOT}/backup.json:/app/backup.json" \
  -v "${PROJECT_ROOT}/subscribers.json:/app/subscribers.json" \
  "${IMAGE_NAME}" > "${LOG_FILE}"

# 从 docker run 输出中读取容器 ID
CONTAINER_ID=$(cat "${LOG_FILE}")
echo "📦 容器 ID: ${CONTAINER_ID}"

# 将容器日志实时追加写入日志文件（后台运行）
docker logs -f "${CONTAINER_ID}" >> "${LOG_FILE}" 2>&1 &
