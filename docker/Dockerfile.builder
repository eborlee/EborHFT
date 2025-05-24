# ==== 基础镜像：CentOS 7 + Rust 1.87.0 + Python 3.8.18 ====
FROM centos:7 as builder

# ==== 配置阿里云源并安装系统构建依赖 ====
RUN curl -o /etc/yum.repos.d/CentOS-Base.repo https://mirrors.aliyun.com/repo/Centos-7.repo && \
    yum clean all && yum makecache && \
    yum install -y \
    gcc gcc-c++ make wget curl openssl-devel zlib-devel \
    bzip2-devel readline-devel sqlite-devel xz-devel \
    libffi-devel ca-certificates tar git && \
    yum clean all

# ==== 安装 Rust 1.87.0（使用 rustup）====
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.87.0
ENV PATH="/root/.cargo/bin:$PATH"

# ==== 安装 Python 3.8.18 from source ====
WORKDIR /opt
RUN wget https://www.python.org/ftp/python/3.8.18/Python-3.8.18.tgz && \
    tar -xzf Python-3.8.18.tgz && \
    cd Python-3.8.18 && \
    ./configure --prefix=/usr/local/python3.8 --without-ensurepip && \
    make -j$(nproc) && \
    make altinstall

# ==== 安装 pip（适配 Python 3.8）并软链接为 pip ====
RUN ln -sf /usr/local/python3.8/bin/python3.8 /usr/bin/python3 && \
    ln -sf /usr/local/python3.8/bin/python3.8 /usr/bin/python && \
    curl -O https://bootstrap.pypa.io/pip/3.8/get-pip.py && \
    python3 get-pip.py && \
    ln -sf /usr/local/python3.8/bin/pip3 /usr/bin/pip

# ==== 安装 Python 项目依赖 ====
WORKDIR /app
COPY scripts/requirements.txt scripts/requirements.txt
RUN python -m pip install --no-cache-dir -r scripts/requirements.txt

# ==== 可选：输出版本信息确认 ====
RUN rustc --version && python --version
