# PBFT 共识区块链项目文档

## 概述

本项目是基于 PBFT（Practical Byzantine Fault Tolerance）共识算法实现的区块链系统，采用 Rust 语言开发。系统包含节点通信、共识流程、状态管理、数据持久化等核心模块，支持拜占庭容错、视图切换、心跳检测等特性。

## 模块说明

### 1. Client 模块 (`src/client.rs`)

- **功能**：管理节点运行时信息

- 核心结构：

  ```
  pub struct Client {
      pub local_node_id: u64,
      pub local_udp_socket: UdpSocket,
      pub private_key: RsaPrivateKey,
      pub public_key: RsaPublicKey,
      pub identities: Vec<Identity>,
      pub nodes_number: u64,
  }
  ```

- 主要方法：

  - `new()`: 初始化客户端
  - `is_primary()`: 判断是否为主节点

### 2. Config 模块 (`src/config.rs`)

- **功能**：管理三类配置
- 配置类型：
  1. Identity：节点身份信息（IP/端口/公钥）
  2. ConstantConfig：持久化配置（数据库名/组播地址等）
  3. VariableConfig：动态配置（当前视图编号）

### 3. Key 模块 (`src/key.rs`)

- **功能**：密码学操作
- 核心能力：
  - 加载 RSA 密钥对
  - 对 11 种消息类型进行签名/验证
  - 支持 SHA256 哈希算法

### 4. Network 模块 (`src/network.rs`)

- **功能**：网络通信

- 核心方法：

  ```
  pub async fn send_udp_data(
      socket: &UdpSocket,
      target: &SocketAddr,
      msg_type: MessageType,
      content: &[u8]
  ) -> Result<()>
  ```

### 5. PBFT 状态机 (`src/pbft.rs`)

- **共识状态**：

  ```
  pub enum Step {
      ReceivingViewResponse,
      ReceivingStateResponse,
      ReceivingSyncResponse,
      NoPrimary,
      ReceivingViewChange,
      Ok,
      ReceivingPrepare,
      ReceivingCommit
  }
  ```

- 状态转换：

  - 维护视图编号、序列号
  - 管理 Prepare/Commit 阶段的节点投票

### 6. 区块链存储 (`src/store.rs`)

- **数据结构**：

  ```
  pub struct Block {
      pub index: u64,
      pub timestamp: u64,
      pub transactions: Vec<Transaction>,
      pub previous_hash: Vec<u8>,
      pub hash: Vec<u8>
  }
  ```

- 存储特性：

  - 使用 RocksDB 持久化
  - 支持区块范围查询
  - 自动生成创世区块

## 核心流程

### PBFT 共识流程

1. **请求阶段**：
   - 客户端发送 Request
   - 主节点验证后广播 PrePrepare
2. **准备阶段**：
   - 节点验证 PrePrepare 后广播 Prepare
   - 收集 2f+1 个 Prepare 进入 Commit 阶段
3. **提交阶段**：
   - 节点广播 Commit
   - 收集 2f+1 个 Commit 后写入区块

### 视图切换流程

1. 从节点检测主节点超时
2. 发起 ViewChange 广播
3. 收集 2f+1 个 ViewChange
4. 切换视图并同步状态

### 心跳机制

- 主节点每秒广播心跳
- 从节点通过心跳重置视图切换计时器

## 配置说明

### 环境变量

```
local_node_id=0
identity_config_path=config/identity.json
constant_config_path=config/constant.json
variable_config_path=config/variable.json
private_key_path=keys/private.pem
public_key_path=keys/public.pem
```

### 配置文件示例

`identity.json`:

```
[
  {
    "node_id": 0,
    "ip": "127.0.0.1",
    "port": 8000,
    "public_key": "-----BEGIN RSA PUBLIC KEY-----\n..."
  }
]
```

`constant.json`:

```
{
  "database_name": "blockchain_db",
  "multi_cast_addr": "224.0.0.1:5000",
  "block_size": 100
}
```

## 运行指南

### 编译运行

1. 安装依赖：

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

1. 生成密钥对：

```
openssl genrsa -out private.pem 2048
openssl rsa -in private.pem -pubout -out public.pem
```

1. 启动节点：

```
cargo run
```

### 命令行操作

```
# 查询最新区块
last

# 查询指定区块
index 5

# 发送测试交易
test 1000 500  # 发送1000笔交易，间隔500μs
```

## 性能指标

- TPS 计算公式：

  ```
  TPS = (end_index - start_index) * tx_per_block / (end_time - start_time)
  ```

- 测试输出示例：

  ```
  begin_index: 1024, end_index: 1324
  blocksize: 100
  tps = 3250.67
  ```

## 注意事项

1. 确保组播地址在局域网内可用
2. 不同节点的 identity.json 需要包含完整的节点列表
3. 视图切换超时时间硬编码为 2 秒（可调整）
4. 使用前需正确配置防火墙规则

## 架构图

复制

```
+------------+      +-------------+      +-----------+
|  Client    |<---->| PBFT State  |<---->| Network   |
+------------+      +-------------+      +-----------+
                       |    |
                       v    v
                 +------------+      +----------+
                 | BlockStore |<---->| RocksDB  |
                 +------------+      +----------+
```

本系统实现了 PBFT 共识的核心逻辑，具备拜占庭容错能力，适用于需要高一致性的分布式场景。开发者可通过扩展 Transaction 类型和调整配置参数适配不同应用需求。
