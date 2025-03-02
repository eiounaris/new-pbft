use crate::config::Identity;

use rsa::{RsaPrivateKey, RsaPublicKey};
use tokio::net::UdpSocket;

/// 节点运行信息
pub struct Client {
    pub local_node_id: u64,
    pub local_udp_socket: UdpSocket,
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
    pub identities: Vec<Identity>,
    pub nodes_number: u64,
}
impl Client {
    /// 初始化构造函数
    pub fn new(
        local_node_id: u64,
        local_udp_socket: UdpSocket,
        private_key: RsaPrivateKey,
        public_key: RsaPublicKey,
        identities: Vec<Identity>,
    ) -> Self {
        let nodes_number = identities.len() as u64;
        Client {
            local_node_id,
            local_udp_socket,
            private_key,
            public_key,
            identities,
            nodes_number,
        }
    }

    /// 判断是否为主节点
    pub fn is_primarry(&self, view_number: u64) -> bool {
        view_number % self.nodes_number == self.local_node_id
    }
}