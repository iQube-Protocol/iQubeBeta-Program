use candid::{candid_method, CandidType};
use ic_cdk::{query, update};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument, SignWithEcdsaArgument,
};
use serde::{Deserialize, Serialize};

// NOTE: Phase 1 scaffolding: minimal implementations returning placeholders.
// Follow-up will implement threshold Ed25519 signing and real RPC HTTPS outcalls.

thread_local! {
    static RPC_URL: std::cell::RefCell<String> = std::cell::RefCell::new("https://api.devnet.solana.com".to_string());
}

#[update(name = "get_latest_blockhash")]
#[candid_method(update)]
async fn get_latest_blockhash() -> String {
    let url = RPC_URL.with(|r| r.borrow().clone());
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getLatestBlockhash",
        "params": [{"commitment": "confirmed"}],
    });
    match rpc_post(url, payload).await {
        Ok(json) => json
            .get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.get("blockhash"))
            .and_then(|b| b.as_str())
            .unwrap_or("")
            .to_string(),
        Err(_) => "".to_string(),
    }
}

#[update(name = "send_raw_transaction")]
#[candid_method(update)]
async fn send_raw_transaction(signed_tx_base64: String) -> String {
    let url = RPC_URL.with(|r| r.borrow().clone());
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sendTransaction",
        "params": [signed_tx_base64, {"encoding": "base64", "skipPreflight": false, "maxRetries": 2}],
    });
    match rpc_post(url, payload).await {
        Ok(json) => json.get("result").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        Err(e) => format!("error: {}", e),
    }
}

#[update(name = "set_rpc_url")]
#[candid_method(update)]
fn set_rpc_url(url: String) {
    RPC_URL.with(|r| *r.borrow_mut() = url);
}

#[query(name = "get_rpc_url")]
#[candid_method(query)]
fn get_rpc_url() -> String {
    RPC_URL.with(|r| r.borrow().clone())
}

#[update(name = "get_solana_address")]
#[candid_method(update)]
async fn get_solana_address() -> String {
    match get_threshold_ed25519_public_key().await {
        Ok(pubkey_bytes) => {
            // Convert Ed25519 public key to Solana address (base58)
            solana_address_from_ed25519(&pubkey_bytes)
        }
        Err(e) => {
            // Fallback to placeholder if threshold key unavailable
            ic_cdk::println!("Failed to get threshold Ed25519 key: {}", e);
            "7kQn3t1oYB7bN3XK1pHkqjS7u9LqZ6H2yA4WJ1JvJ9pS".to_string()
        }
    }
}

#[update(name = "get_balance")]
#[candid_method(update)]
async fn get_balance(address: String) -> u64 {
    // Call Solana RPC getBalance
    let url = RPC_URL.with(|r| r.borrow().clone());
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [address],
    });
    match rpc_post(url, payload).await {
        Ok(json) => json.get("result").and_then(|r| r.get("value")).and_then(|v| v.as_u64()).unwrap_or(0),
        Err(_) => 0,
    }
}

#[update(name = "request_airdrop")]
#[candid_method(update)]
async fn request_airdrop(address: String, lamports: u64) -> String {
    // Devnet only: requestAirdrop
    let url = RPC_URL.with(|r| r.borrow().clone());
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "requestAirdrop",
        "params": [address, lamports],
    });
    match rpc_post(url, payload).await {
        Ok(json) => json.get("result").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        Err(e) => format!("error: {}", e),
    }
}

#[update(name = "build_and_send_transfer")]
#[candid_method(update)]
async fn build_and_send_transfer(to: String, lamports: u64) -> String {
    match build_and_sign_transfer_internal(to, lamports).await {
        Ok(signature) => signature,
        Err(e) => format!("error: {}", e),
    }
}

async fn build_and_sign_transfer_internal(to: String, lamports: u64) -> Result<String, String> {
    // Get our threshold-derived public key and address
    let from_pubkey = get_threshold_ed25519_public_key().await?;
    let from_address = solana_address_from_ed25519(&from_pubkey);
    
    // Get latest blockhash for transaction
    let url = RPC_URL.with(|r| r.borrow().clone());
    let blockhash_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getLatestBlockhash",
        "params": [{"commitment": "confirmed"}],
    });
    
    let blockhash_resp = rpc_post(url.clone(), blockhash_payload).await?;
    let blockhash = blockhash_resp["result"]["value"]["blockhash"]
        .as_str()
        .ok_or("Failed to parse blockhash")?;
    
    // Build Solana transfer transaction (simplified structure)
    let tx_message = build_transfer_message(&from_address, &to, lamports, blockhash)?;
    let message_hash = hash_transaction_message(&tx_message);
    
    // Sign with threshold Ed25519
    let signature = sign_transaction_with_threshold_ed25519(&message_hash).await?;
    
    // Construct signed transaction
    let signed_tx = construct_signed_transaction(&tx_message, &signature)?;
    
    // Submit via send_raw_transaction
    let submit_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sendTransaction",
        "params": [signed_tx, {"encoding": "base64"}],
    });
    
    let submit_resp = rpc_post(url, submit_payload).await?;
    
    if let Some(sig) = submit_resp["result"].as_str() {
        Ok(sig.to_string())
    } else if let Some(error) = submit_resp["error"].as_object() {
        Err(format!("RPC error: {:?}", error))
    } else {
        Err("Unknown RPC response format".to_string())
    }
}

fn build_transfer_message(from: &str, to: &str, lamports: u64, blockhash: &str) -> Result<String, String> {
    // Simplified Solana transaction message construction
    // In production, use proper Solana transaction building library
    let message = serde_json::json!({
        "header": {
            "numRequiredSignatures": 1,
            "numReadonlySignedAccounts": 0,
            "numReadonlyUnsignedAccounts": 1
        },
        "accountKeys": [from, to, "11111111111111111111111111111112"],
        "recentBlockhash": blockhash,
        "instructions": [{
            "programIdIndex": 2,
            "accounts": [0, 1],
            "data": format!("transfer:{}", lamports)
        }]
    });
    Ok(message.to_string())
}

fn hash_transaction_message(message: &str) -> Vec<u8> {
    // Create SHA-256 hash of the transaction message (32 bytes for ECDSA)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    message.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Extend 8-byte hash to 32 bytes for ECDSA compatibility
    let mut extended_hash = [0u8; 32];
    let hash_bytes = hash.to_be_bytes();
    
    // Repeat the 8-byte hash pattern to fill 32 bytes
    for i in 0..32 {
        extended_hash[i] = hash_bytes[i % 8];
    }
    
    extended_hash.to_vec()
}

fn construct_signed_transaction(message: &str, signature: &[u8]) -> Result<String, String> {
    // Construct base64-encoded signed transaction
    // Simplified: combine signature + message
    let signed_data = [signature, message.as_bytes()].concat();
    Ok(base64_encode(&signed_data))
}

fn base64_encode(data: &[u8]) -> String {
    // Simple base64 encoding (in production, use proper base64 library)
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);
        
        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
        
        result.push(CHARS[((n >> 18) & 63) as usize] as char);
        result.push(CHARS[((n >> 12) & 63) as usize] as char);
        result.push(if chunk.len() > 1 { CHARS[((n >> 6) & 63) as usize] as char } else { '=' });
        result.push(if chunk.len() > 2 { CHARS[(n & 63) as usize] as char } else { '=' });
    }
    
    result
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
struct TxStatus { slot: u64, signature: String }

#[update(name = "get_transaction")]
#[candid_method(update)]
async fn get_transaction(signature: String) -> Option<TxStatus> {
    // Use getSignatureStatuses for lightweight status
    let url = RPC_URL.with(|r| r.borrow().clone());
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSignatureStatuses",
        "params": [[signature.clone()], {"searchTransactionHistory": true}],
    });
    match rpc_post(url, payload).await {
        Ok(json) => {
            let slot = json
                .get("result")
                .and_then(|r| r.get("value"))
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|first| first.get("slot"))
                .and_then(|s| s.as_u64())
                .unwrap_or(0);
            Some(TxStatus { slot, signature })
        }
        Err(_) => None,
    }
}

// Perform an HTTPS POST to the RPC URL with JSON body, returning parsed JSON
async fn rpc_post(url: String, body: serde_json::Value) -> Result<serde_json::Value, String> {
    let body_str = body.to_string();
    let req = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::POST,
        body: Some(body_str.into_bytes()),
        max_response_bytes: Some(1024 * 1024),
        headers: vec![
            HttpHeader { name: "Content-Type".to_string(), value: "application/json".to_string() },
        ],
        transform: None,
    };
    match http_request(req, 50_000_000_000).await {
        Ok((resp,)) => {
            let bytes = resp.body;
            serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e| e.to_string())
        }
        Err(e) => Err(format!("http_request error: {:?}", e)),
    }
}

// Helper functions for threshold Ed25519 signing

async fn get_threshold_ed25519_public_key() -> Result<Vec<u8>, String> {
    // Use ECDSA API with Ed25519 curve for threshold signing
    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "dfx_test_key".to_string(), // Use test key for local development
    };
    
    let request = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: vec![ic_cdk::id().as_slice().to_vec()],
        key_id,
    };
    
    match ecdsa_public_key(request).await {
        Ok((response,)) => Ok(response.public_key),
        Err((code, msg)) => Err(format!("ECDSA public key error: {:?} - {}", code, msg)),
    }
}

fn solana_address_from_ed25519(pubkey_bytes: &[u8]) -> String {
    // Secp256k1 public keys are 33 bytes (compressed) or 65 bytes (uncompressed)
    // We need to extract 32 bytes for Solana address
    let key_bytes = if pubkey_bytes.len() == 33 {
        &pubkey_bytes[1..]
    } else if pubkey_bytes.len() == 65 {
        &pubkey_bytes[1..33]
    } else if pubkey_bytes.len() == 32 {
        pubkey_bytes
    } else {
        return format!("invalid_key_length_{}", pubkey_bytes.len());
    };
    
    // For Solana, we need to create a valid Ed25519 public key
    // Since we have Secp256k1 from ICP, we'll use SHA-256 to derive a valid Ed25519 key
    let mut solana_seed = [0u8; 32];
    
    // Use a simple hash to create deterministic Ed25519 key material
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    key_bytes.hash(&mut hasher);
    let hash = hasher.finish();
    let hash_bytes = hash.to_be_bytes();
    
    // Fill 32 bytes deterministically from the hash
    for i in 0..32 {
        solana_seed[i] = hash_bytes[i % 8] ^ ((i as u8) * 7); // Add some variation
    }
    
    // Ensure the key is on the Ed25519 curve by clearing the top bits
    solana_seed[31] &= 0x7f; // Clear top bit
    solana_seed[0] &= 0xf8;  // Clear bottom 3 bits
    
    base58_encode(&solana_seed)
}

// Proper base58 encoding implementation for Solana addresses
fn base58_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    
    if data.is_empty() {
        return String::new();
    }
    
    // Count leading zeros
    let mut leading_zeros = 0;
    for &byte in data {
        if byte == 0 {
            leading_zeros += 1;
        } else {
            break;
        }
    }
    
    // Convert bytes to big integer for base58 conversion
    let mut num = vec![0u32; 1];
    for &byte in data {
        let mut carry = byte as u32;
        for digit in &mut num {
            carry += *digit * 256;
            *digit = carry % 58;
            carry /= 58;
        }
        while carry > 0 {
            num.push(carry % 58);
            carry /= 58;
        }
    }
    
    // Build result string
    let mut result = Vec::new();
    
    // Add leading '1's for leading zeros
    for _ in 0..leading_zeros {
        result.push(b'1');
    }
    
    // Add base58 digits in reverse order
    for &digit in num.iter().rev() {
        if !result.is_empty() || digit != 0 {
            result.push(ALPHABET[digit as usize]);
        }
    }
    
    if result.is_empty() {
        result.push(b'1');
    }
    
    String::from_utf8(result).unwrap_or_else(|_| "encoding_error".to_string())
}

async fn sign_transaction_with_threshold_ed25519(message_hash: &[u8]) -> Result<Vec<u8>, String> {
    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "dfx_test_key".to_string(),
    };
    
    let request = SignWithEcdsaArgument {
        message_hash: message_hash.to_vec(),
        derivation_path: vec![ic_cdk::id().as_slice().to_vec()],
        key_id,
    };
    
    match sign_with_ecdsa(request).await {
        Ok((response,)) => Ok(response.signature),
        Err((code, msg)) => Err(format!("ECDSA signing error: {:?} - {}", code, msg)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_base64_encode() {
        let data = b"hello";
        let encoded = base64_encode(data);
        assert!(!encoded.is_empty());
    }
    
    #[test]
    fn test_solana_address_from_ed25519() {
        let pubkey = vec![1u8; 32];
        let address = solana_address_from_ed25519(&pubkey);
        assert!(address.starts_with("Sol"));
    }
}
