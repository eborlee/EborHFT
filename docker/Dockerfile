# === 编译阶段 ===
FROM eborhft-builder:stable as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin trade_monitor

# === 运行阶段（干净运行环境）===
FROM centos:7
RUN curl -o /etc/yum.repos.d/CentOS-Base.repo https://mirrors.aliyun.com/repo/Centos-7.repo && \
    yum clean all && yum makecache

# 安装运行期必要依赖
RUN yum install -y ca-certificates && yum clean all

# 拷贝 Python3.8 解释器及 site-packages（不含 pip）
COPY --from=builder /usr/local/python3.8 /usr/local/python3.8
COPY --from=builder /usr/local/lib/python3.8 /usr/local/lib/python3.8
RUN ln -sf /usr/local/python3.8/bin/python3.8 /usr/bin/python3

# 设置工作目录
WORKDIR /app

# 拷贝编译产物、配置、数据和脚本
COPY --from=builder /app/target/release/trade_monitor /app/trade_monitor
COPY --from=builder /app/config.toml /app/
# COPY --from=builder /app/data /app/data
COPY --from=builder /app/scripts /app/scripts

# 容器入口
CMD ["./trade_monitor"]
