use super::config::Identity;

use rsa::{RsaPrivateKey, RsaPublicKey};
use tokio::net::UdpSocket;

use std::sync::Arc;


/// 节点运行信息
pub struct Client {
    pub local_node_id: u64,
    pub local_udp_socket: Arc<UdpSocket>,
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
    pub identities: Vec<Identity>,
}
impl Client {
    /// 初始化构造函数
    pub fn new(
        local_node_id: u64,
        local_udp_socket: Arc<UdpSocket>,
        private_key: RsaPrivateKey,
        public_key: RsaPublicKey,
        identities: Vec<Identity>,
    ) -> Self {
        Client {
            local_node_id,
            local_udp_socket,
            private_key,
            public_key,
            identities,
        }
    }

    /// 判断是否为主节点
    pub fn is_primarry(&self, view_number: u64) -> bool {
        view_number % self.identities.len() as u64 == self.local_node_id
    }
}