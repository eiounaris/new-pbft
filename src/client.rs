use super::config::Identity;

use rsa::{RsaPrivateKey, RsaPublicKey};

/// 存储节点运行不变配置信息
pub struct Client {
    pub local_node_id: u64,
    pub local_socket_addr: std::net::SocketAddr,
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
    pub all_indetitys: Vec<Identity>,
}