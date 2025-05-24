# === 编译阶段：用预构建 builder 镜像（含 Rust + Python3.8） ===
FROM eborhft-builder:stable as builder

WORKDIR /app

# 拷贝你完整的项目代码（含 Cargo.toml, src/, config, scripts）
COPY . .

# 构建 Release 版本
RUN cargo build --release

# === 运行阶段：CentOS + Python3.8（如运行时还需调用 Python 脚本）===
FROM centos:7

# 安装最低依赖（如系统证书）
RUN yum install -y ca-certificates && yum clean all

# 可选：复制 Python3.8 解释器（如果运行脚本仍需 python3.8）
# 你可以从 builder 镜像中 copy Python 本体（如下）
COPY --from=builder /usr/local/python3.8 /usr/local/python3.8
RUN ln -sf /usr/local/python3.8/bin/python3.8 /usr/bin/python3 && \
    ln -sf /usr/local/bin/pip3 /usr/bin/pip3

# 复制 Python 包（如安装在 builder 中的 site-packages）
COPY --from=builder /usr/local/lib/python3.8 /usr/local/lib/python3.8

# 设置工作目录
WORKDIR /app

# 一次性复制编译产物和所有资源  构建阶段builder 镜像 -> 运行阶段centos 镜像
COPY --from=builder /app /app
# 启动程序
CMD ["./trade_monitor"]
