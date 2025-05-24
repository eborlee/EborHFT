# ==== 基础镜像：Rust 1.87.0 官方稳定版本 ====
FROM rust:1.87.0 as builder

# ==== 安装系统依赖 ====
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential wget curl libssl-dev zlib1g-dev \
    libbz2-dev libreadline-dev libsqlite3-dev xz-utils \
    libffi-dev ca-certificates && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# ==== 安装 Python 3.8.18 from source ====
WORKDIR /opt
RUN wget https://www.python.org/ftp/python/3.8.18/Python-3.8.18.tgz && \
    tar -xzf Python-3.8.18.tgz && \
    cd Python-3.8.18 && \
    ./configure --prefix=/usr/local/python3.8 --without-ensurepip && \
    make -j$(nproc) && \
    make altinstall

# ==== 配置 python3 / pip3 ====
RUN ln -sf /usr/local/python3.8/bin/python3.8 /usr/bin/python3 && \
    curl -O https://bootstrap.pypa.io/get-pip.py && \
    python3 get-pip.py && \
    ln -sf /usr/local/bin/pip3 /usr/bin/pip3

# ==== 拷贝并安装 Python 项目依赖 ====
WORKDIR /app
COPY scripts/requirements.txt scripts/requirements.txt
RUN pip3 install --no-cache-dir -r scripts/requirements.txt

# ==== 可选版本信息输出 ====
RUN rustc --version && python3 --version && pip3 --version
