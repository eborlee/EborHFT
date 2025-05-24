# === 编译阶段 ===
FROM eborhft-builder:stable as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin trade_monitor || true
COPY . .
RUN cargo build --release --bin trade_monitor

# === 运行阶段（干净运行环境）===
FROM centos:7
RUN curl -o /etc/yum.repos.d/CentOS-Base.repo https://mirrors.aliyun.com/repo/Centos-7.repo && \
    yum clean all && yum makecache

# 安装运行期必要依赖
RUN yum install -y ca-certificates && yum clean all

# 拷贝 Python 解释器及完整环境（含 site-packages）
COPY --from=builder /usr/local/python3.8 /usr/local/python3.8
# 创建软链接，方便容器内调用 python3
RUN ln -sf /usr/local/python3.8/bin/python3.8 /usr/bin/python3

# 设置工作目录
WORKDIR /app

# 拷贝编译产物、配置、数据和脚本
COPY --from=builder /app/target/release/trade_monitor /app/trade_monitor
# COPY --from=builder /app/config.toml /app/
# COPY --from=builder /app/data /app/data
# COPY --from=builder /app/scripts /app/scripts

# 容器入口
CMD ["./trade_monitor"]
