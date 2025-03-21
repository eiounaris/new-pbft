// src/key.rs
use crate::message::{
    Request, 
    PrePrepare, 
    Prepare, 
    Commit, 
    Hearbeat, 
    ViewChange, 
    NewView, 
    ViewResponse, 
    StateResponse, 
    SyncResponse
};


use rsa::{pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey}, Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};
use tokio::fs;

// ---

/// 加载私钥
pub async fn load_private_key(file_path: &str) -> Result<RsaPrivateKey, String> {
    let pem = fs::read_to_string(file_path).await
        .map_err(|e| e.to_string())?;
    RsaPrivateKey::from_pkcs1_pem(&pem)
        .map_err(|e| e.to_string())
}

/// 加载公钥
pub async fn load_public_key(file_path: &str) -> Result<RsaPublicKey, String> {
    let pem = fs::read_to_string(file_path).await
        .map_err(|e| e.to_string())?;
    RsaPublicKey::from_pkcs1_pem(&pem)
        .map_err(|e| e.to_string())
}

// ---

/// 使用私钥签名数据
fn sign_data(private_key: &RsaPrivateKey, data: &[u8]) -> Result<Vec<u8>, String> {
    let hashed_data = Sha256::digest(data);
    private_key.sign(Pkcs1v15Sign::new::<Sha256>(), &hashed_data)
        .map_err(|e| e.to_string())
}

/// 使用公钥验证签名
fn verify_signature(public_key: &RsaPublicKey, data: &[u8], signature: &[u8]) -> Result<bool, String> {
    let hashed_data = Sha256::digest(data);
    if let Ok(_) = public_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_data, &signature) {
        return Ok(true)
    }
    Ok(false)
}

// ---

/// 签名请求消息
pub fn sign_request(priv_key: &RsaPrivateKey, request: &mut Request) -> Result<(), String> {
    request.signature = Vec::new();
    request.signature = sign_data(priv_key, &bincode::serialize(&request)
        .map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证请求消息
pub fn verify_request(public_key: &RsaPublicKey, request: &mut Request) -> Result<bool, String> {
    let signature = request.signature.clone();
    request.signature = Vec::new();
    let bincode_data = &bincode::serialize(&request)
        .map_err(|e| e.to_string())?;
    request.signature = signature;
    verify_signature(public_key, &bincode_data, &request.signature)
}

// ---

/// 签名预准备消息
pub fn sign_preprepare(priv_key: &RsaPrivateKey, preprepare: &mut PrePrepare) -> Result<(), String> {
    preprepare.signature = Vec::new();
    preprepare.signature = sign_data(priv_key, &bincode::serialize(&preprepare)
        .map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证预准备消息
pub fn verify_preprepare(public_key: &RsaPublicKey, preprepare: &mut PrePrepare) -> Result<bool, String> {
    let signature = preprepare.signature.clone();
    preprepare.signature = Vec::new();
    let bincode_data = &bincode::serialize(&preprepare)
        .map_err(|e| e.to_string())?;
    preprepare.signature = signature;
    verify_signature(public_key, &bincode_data, &preprepare.signature)
}

// ---

/// 签名准备消息
pub fn sign_prepare(priv_key: &RsaPrivateKey, prepare: &mut Prepare) -> Result<(), String> {
    prepare.signature = Vec::new();
    prepare.signature = sign_data(priv_key, &bincode::serialize(&prepare)
        .map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证准备消息
pub fn verify_prepare(public_key: &RsaPublicKey, prepare: &mut Prepare) -> Result<bool, String> {
    let signature = prepare.signature.clone();
    prepare.signature = Vec::new();
    let bincode_data = &bincode::serialize(&prepare)
        .map_err(|e| e.to_string())?;
    prepare.signature = signature;
    verify_signature(public_key, &bincode_data, &prepare.signature)
}

// ---

/// 签名提交消息
pub fn sign_commit(priv_key: &RsaPrivateKey, commit: &mut Commit) -> Result<(), String> {
    commit.signature = Vec::new();
    commit.signature = sign_data(priv_key, &bincode::serialize(&commit)
        .map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证提交消息
pub fn verify_commit(public_key: &RsaPublicKey, commit: &mut Commit) -> Result<bool, String> {
    let signature = commit.signature.clone();
    commit.signature = Vec::new();
    let bincode_data = &bincode::serialize(&commit)
        .map_err(|e| e.to_string())?;
    commit.signature = signature;
    verify_signature(public_key, &bincode_data, &commit.signature)
}

// ---

/// 签名心跳消息
pub fn sign_heartbeat(priv_key: &RsaPrivateKey, hearbeat: &mut Hearbeat) -> Result<(), String> {
    hearbeat.signature = Vec::new();
    hearbeat.signature = sign_data(priv_key, &bincode::serialize(&hearbeat)
        .map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证心跳消息
pub fn verify_heartbeat(public_key: &RsaPublicKey, hearbeat: &mut Hearbeat) -> Result<bool, String> {
    let signature = hearbeat.signature.clone();
    hearbeat.signature = Vec::new();
    let bincode_data = &bincode::serialize(&hearbeat)
        .map_err(|e| e.to_string())?;
    hearbeat.signature = signature;
    verify_signature(public_key, &bincode_data, &hearbeat.signature)
}

// ---

/// 签名试图切换消息
pub fn sign_view_change(priv_key: &RsaPrivateKey, view_change: &mut ViewChange) -> Result<(), String> {
    view_change.signature = Vec::new();
    view_change.signature = sign_data(priv_key, &bincode::serialize(&view_change)
        .map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证图切换消息
pub fn verify_view_change(public_key: &RsaPublicKey, view_change: &mut ViewChange) -> Result<bool, String> {
    let signature = view_change.signature.clone();
    view_change.signature = Vec::new();
    let bincode_data = &bincode::serialize(&view_change)
        .map_err(|e| e.to_string())?;
    view_change.signature = signature;
    verify_signature(public_key, &bincode_data, &view_change.signature)
}

// ---

/// 签名新视图消息
pub fn sign_new_view(priv_key: &RsaPrivateKey, new_view: &mut NewView) -> Result<(), String> {
    new_view.signature = Vec::new();
    new_view.signature = sign_data(priv_key, &bincode::serialize(&new_view)
        .map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证新视图消息
pub fn verify_new_view(public_key: &RsaPublicKey, new_view: &mut NewView) -> Result<bool, String> {
    let signature = new_view.signature.clone();
    new_view.signature = Vec::new();
    let bincode_data = &bincode::serialize(&new_view)
        .map_err(|e| e.to_string())?;
    new_view.signature = signature;
    verify_signature(public_key, &bincode_data, &new_view.signature)
}

// ---

/// 签名视图响应消息
pub fn sign_view_response(priv_key: &RsaPrivateKey, view_response: &mut ViewResponse) -> Result<(), String> {
    view_response.signature = Vec::new();
    view_response.signature = sign_data(priv_key, &bincode::serialize(&view_response)
        .map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证视图响应消息
pub fn verify_view_response(public_key: &RsaPublicKey, view_response: &mut ViewResponse) -> Result<bool, String> {
    let signature = view_response.signature.clone();
    view_response.signature = Vec::new();
    let bincode_data = &bincode::serialize(&view_response)
        .map_err(|e| e.to_string())?;
    view_response.signature = signature;
    verify_signature(public_key, &bincode_data, &view_response.signature)
}

// ---

/// 签名状态响应消息
pub fn sign_state_response(priv_key: &RsaPrivateKey, state_response: &mut StateResponse) -> Result<(), String> {
    state_response.signature = Vec::new();
    state_response.signature = sign_data(priv_key, &bincode::serialize(&state_response)
        .map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证状态响应消息
pub fn verify_state_response(public_key: &RsaPublicKey, state_response: &mut StateResponse) -> Result<bool, String> {
    let signature = state_response.signature.clone();
    state_response.signature = Vec::new();
    let bincode_data = &bincode::serialize(&state_response)
        .map_err(|e| e.to_string())?;
    state_response.signature = signature;
    verify_signature(public_key, &bincode_data, &state_response.signature)
}

// ---

/// 签名同步响应消息
pub fn sign_sync_response(priv_key: &RsaPrivateKey, sync_response: &mut SyncResponse) -> Result<(), String> {
    sync_response.signature = Vec::new();
    sync_response.signature = sign_data(priv_key, &bincode::serialize(&sync_response)
        .map_err(|e| e.to_string())?)?;
    Ok(())
}

/// 验证同步响应消息
pub fn verify_sync_response(public_key: &RsaPublicKey, sync_response: &mut SyncResponse) -> Result<bool, String> {
    let signature = sync_response.signature.clone();
    sync_response.signature = Vec::new();
    let bincode_data = &bincode::serialize(&sync_response)
        .map_err(|e| e.to_string())?;
    sync_response.signature = signature;
    verify_signature(public_key, &bincode_data, &sync_response.signature)
}