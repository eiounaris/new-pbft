#![allow(dead_code, unused_variables)]


use crate::message::MessageType;

use tokio::net::UdpSocket;

use std::net::SocketAddr;


/// 发送UDP数据
pub async fn send_udp_data(local_udp_socket: &UdpSocket, target_udp_socket: &SocketAddr, message_type: MessageType, content: &[u8]) {
    let mut message = Box::new(Vec::new());
    message.push(message_type as u8);
    message.extend_from_slice(&content);
    local_udp_socket.send_to(&message, target_udp_socket).await.unwrap();
}

