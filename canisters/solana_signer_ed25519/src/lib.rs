use candid::{candid_method, CandidType, Func};
use ic_cdk::{query, update};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformArgs, TransformContext,
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

#[query(name = "get_solana_address")]
#[candid_method(query)]
fn get_solana_address() -> String {
    // Placeholder devnet address (base58) - replace with threshold Ed25519 pubkey derivation
    "7kQn3t1oYB7bN3XK1pHkqjS7u9LqZ6H2yA4WJ1JvJ9pS".to_string()
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
fn build_and_send_transfer(_to: String, _lamports: u64) -> String {
    // Placeholder signature
    "transfer_sig_placeholder".to_string()
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

// no transform needed; responses parsed directly

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smoke() {
        assert!(!get_solana_address().is_empty());
    }
}
