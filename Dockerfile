# 使用兼容性更好的基础镜像
FROM debian

# 设置工作目录并创建必要目录
WORKDIR /app
RUN mkdir -p /state

# 复制可执行文件并设置权限
COPY new_pbft .
RUN chmod +x /app/new_pbft

# 声明数据卷和端口
VOLUME ["/data"]
EXPOSE 8888

# 入口点配置
ENTRYPOINT ["./new_pbft"]