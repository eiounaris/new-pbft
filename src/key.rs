use super::message::*;


use rsa::{pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey}, Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
use sha2::{Sha256, Digest};


use std::{fs::File, io::Read};


/// 加载私钥
pub fn load_private_key(file_path: &str) -> Result<RsaPrivateKey, String> {
    let mut file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut pem = String::new();
    file.read_to_string(&mut pem).map_err(|e| e.to_string())?;
    Ok(RsaPrivateKey::from_pkcs1_pem(&pem).map_err(|e| e.to_string())?)
}

/// 加载公钥
pub fn load_public_key(file_path: &str) -> Result<RsaPublicKey, String> {
    let mut file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut pem = String::new();
    file.read_to_string(&mut pem).map_err(|e| e.to_string())?;
    Ok(RsaPublicKey::from_pkcs1_pem(&pem).map_err(|e| e.to_string())?)
}


/// 使用私钥签名数据
fn sign_data(private_key: &RsaPrivateKey, data: &[u8]) -> Result<Vec<u8>, String> {
    let hashed_data = Sha256::digest(data);
    private_key.sign(Pkcs1v15Sign::new::<Sha256>(), &hashed_data).map_err(|e| e.to_string())
}

/// 使用公钥验证签名
fn verify_signature(pub_key: &RsaPublicKey, data: &[u8], signature: &[u8]) -> Result<bool, String> {
    let hashed_data = Sha256::digest(data);
    pub_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_data, &signature[..]).map_err(|e| e.to_string())?;
    Ok(true)
}

/// 签名请求消息
pub fn sign_request(priv_key: &RsaPrivateKey, request: &mut Request) -> Result<(), String> {
    request.signature = sign_data(priv_key, &bincode::serialize(&request).map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证请求消息
pub fn verify_request(pub_key: &RsaPublicKey, request: &mut Request) -> Result<bool, String> {
    let signature = request.signature.clone();
    request.signature = Vec::new();
    let hashed_data = Sha256::digest(&bincode::serialize(&request).map_err(|e| e.to_string())?);
    pub_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_data, &signature).map_err(|e| e.to_string())?;
    request.signature = signature;
    Ok(true)
}

/// 签名预准备消息
pub fn sign_pre_prepare(priv_key: &RsaPrivateKey, pre_prepare: &mut PrePrepare) -> Result<(), String> {
    pre_prepare.signature = sign_data(priv_key, &bincode::serialize(&pre_prepare).map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证预准备消息
pub fn verify_pre_prepare(pub_key: &RsaPublicKey, pre_prepare: &mut PrePrepare) -> Result<bool, String> {
    let signature = pre_prepare.signature.clone();
    pre_prepare.signature = Vec::new();
    let hashed_data = Sha256::digest(&bincode::serialize(&pre_prepare).map_err(|e| e.to_string())?);
    pub_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_data, &signature[..]).map_err(|e| e.to_string())?;
    pre_prepare.signature = signature;
    Ok(true)
}

/// 签名准备消息
pub fn sign_prepare(priv_key: &RsaPrivateKey, prepare: &mut Prepare) -> Result<(), String> {
    prepare.signature = sign_data(priv_key, &bincode::serialize(&prepare).map_err(|e| e.to_string())?)?;
    Ok(())
}
/// 验证准备消息
pub fn verify_prepare(pub_key: &RsaPublicKey, prepare: &mut Prepare) -> Result<bool, String> {
    let signature = prepare.signature.clone();
    prepare.signature = Vec::new();
    let hashed_data = Sha256::digest(&bincode::serialize(&prepare).map_err(|e| e.to_string())?);
    pub_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_data, &signature[..]).map_err(|e| e.to_string())?;
    prepare.signature = signature;
    Ok(true)
}

/// 签名提交消息
pub fn sign_commit(priv_key: &RsaPrivateKey, commit: &mut Commit) -> Result<(), String> {
    commit.signature = sign_data(priv_key, &bincode::serialize(&commit).map_err(|e| e.to_string())?)?;
    Ok(())
}
/// 验证提交消息
pub fn verify_commit(pub_key: &RsaPublicKey, commit: &mut Commit) -> Result<bool, String> {
    let signature = commit.signature.clone();
    commit.signature = Vec::new();
    let hashed_data = Sha256::digest(&bincode::serialize(&commit).map_err(|e| e.to_string())?);
    pub_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_data, &signature[..]).map_err(|e| e.to_string())?;
    commit.signature = signature;
    Ok(true)
}

/// 签名心跳消息
pub fn sign_heartbeat(priv_key: &RsaPrivateKey, hearbeat: &mut Hearbeat) -> Result<(), String> {
    hearbeat.signature = sign_data(priv_key, &bincode::serialize(&hearbeat).map_err(|e| e.to_string())?)?;
    Ok(())
}
/// 验证心跳消息
pub fn verify_heartbeat(pub_key: &RsaPublicKey, hearbeat: &mut Hearbeat) -> Result<bool, String> {
    let signature = hearbeat.signature.clone();
    hearbeat.signature = Vec::new();
    let hashed_data = Sha256::digest(&bincode::serialize(&hearbeat).map_err(|e| e.to_string())?);
    pub_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_data, &signature[..]).map_err(|e| e.to_string())?;
    hearbeat.signature = signature;
    Ok(true)
}

/// 签名试图切换消息
pub fn sign_view_change(priv_key: &RsaPrivateKey, view_change: &mut ViewChange) -> Result<(), String> {
    view_change.signature = sign_data(priv_key, &bincode::serialize(&view_change).map_err(|e| e.to_string())?)?;
    Ok(())
}
/// 验证图切换消息
pub fn verify_view_change(pub_key: &RsaPublicKey, view_change: &mut ViewChange) -> Result<bool, String> {
    let signature = view_change.signature.clone();
    view_change.signature = Vec::new();
    let hashed_data = Sha256::digest(&bincode::serialize(&view_change).map_err(|e| e.to_string())?);
    pub_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_data, &signature[..]).map_err(|e| e.to_string())?;
    view_change.signature = signature;
    Ok(true)
}

/// 签名新视图消息
pub fn sign_new_view(priv_key: &RsaPrivateKey, new_view: &mut NewView) -> Result<(), String> {
    new_view.signature = sign_data(priv_key, &bincode::serialize(&new_view).map_err(|e| e.to_string())?)?;
    Ok(())
}
/// 验证新视图消息
pub fn verify_new_view(pub_key: &RsaPublicKey, new_view: &mut NewView) -> Result<bool, String> {
    let signature = new_view.signature.clone();
    new_view.signature = Vec::new();
    let hashed_data = Sha256::digest(&bincode::serialize(&new_view).map_err(|e| e.to_string())?);
    pub_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_data, &signature[..]).map_err(|e| e.to_string())?;
    new_view.signature = signature;
    Ok(true)
}