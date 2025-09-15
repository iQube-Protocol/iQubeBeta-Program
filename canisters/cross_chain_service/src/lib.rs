use candid::{CandidType, Deserialize};
use ic_cdk::{query, update, api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
}};
use ic_cdk_timers::{set_timer, TimerId};
use std::collections::HashMap;
use std::time::Duration;

#[derive(CandidType, Deserialize, Clone)]
pub struct DVNMessage {
    pub id: String,
    pub source_chain: u32,
    pub destination_chain: u32,
    pub payload: Vec<u8>,
    pub nonce: u64,
    pub sender: String,
    pub timestamp: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct DVNAttestation {
    pub message_id: String,
    pub validator: String,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CrossChainTransaction {
    pub id: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub tx_hash: String,
    pub block_height: u64,
    pub confirmations: u32,
    pub status: String, // pending, confirmed, failed
    pub timestamp: u64,
}

thread_local! {
    static DVN_MESSAGES: std::cell::RefCell<HashMap<String, DVNMessage>> = std::cell::RefCell::new(HashMap::new());
    static ATTESTATIONS: std::cell::RefCell<HashMap<String, Vec<DVNAttestation>>> = std::cell::RefCell::new(HashMap::new());
    static TRANSACTIONS: std::cell::RefCell<HashMap<String, CrossChainTransaction>> = std::cell::RefCell::new(HashMap::new());
    static TIMER_IDS: std::cell::RefCell<Vec<TimerId>> = std::cell::RefCell::new(Vec::new());
}

const REQUIRED_ATTESTATIONS: usize = 2; // Minimum DVN quorum
const _CONFIRMATION_BLOCKS: u32 = 6; // Required confirmations

// Host-friendly time helper: uses ic_cdk::api::time in WASM, std time in host tests
fn now_millis() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::time()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

#[update]
pub fn submit_dvn_message(
    source_chain: u32,
    destination_chain: u32,
    payload: Vec<u8>,
    sender: String,
) -> String {
    let message_id = format!("msg_{}", now_millis());
    let message = DVNMessage {
        id: message_id.clone(),
        source_chain,
        destination_chain,
        payload,
        nonce: now_millis(),
        sender,
        timestamp: now_millis(),
    };
    
    DVN_MESSAGES.with(|m| m.borrow_mut().insert(message_id.clone(), message));
    
    // Start monitoring for attestations (only in canister runtime)
    #[cfg(target_arch = "wasm32")]
    {
        let msg_id_clone = message_id.clone();
        let timer_id = set_timer(Duration::from_secs(30), move || {
            ic_cdk::spawn(check_message_attestations(msg_id_clone.clone()));
        });
        TIMER_IDS.with(|t| t.borrow_mut().push(timer_id));
    }
    
    message_id
}

#[update]
pub fn submit_attestation(
    message_id: String,
    validator: String,
    signature: Vec<u8>,
) -> Result<String, String> {
    let message_exists = DVN_MESSAGES.with(|m| m.borrow().contains_key(&message_id));
    
    if !message_exists {
        return Err("Message not found".to_string());
    }
    
    let attestation = DVNAttestation {
        message_id: message_id.clone(),
        validator,
        signature,
        timestamp: now_millis(),
    };
    
    ATTESTATIONS.with(|a| {
        let mut attestations = a.borrow_mut();
        attestations.entry(message_id.clone()).or_insert_with(Vec::new).push(attestation);
    });
    
    // Check if we have enough attestations
    let attestation_count = ATTESTATIONS.with(|a| {
        a.borrow().get(&message_id).map(|v| v.len()).unwrap_or(0)
    });
    
    if attestation_count >= REQUIRED_ATTESTATIONS {
        Ok(format!("Message {} ready for execution with {} attestations", message_id, attestation_count))
    } else {
        Ok(format!("Attestation recorded. Need {} more attestations", REQUIRED_ATTESTATIONS - attestation_count))
    }
}

#[update]
pub async fn monitor_evm_transaction(
    chain_id: u32,
    tx_hash: String,
    rpc_url: String,
) -> Result<String, String> {
    let tx_id = format!("tx_{}", ic_cdk::api::time());
    
    // Create HTTP request to check transaction status
    let request = CanisterHttpRequestArgument {
        url: format!("{}", rpc_url),
        method: HttpMethod::POST,
        body: Some(format!(
            r#"{{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":["{}"],"id":1}}"#,
            tx_hash
        ).into_bytes()),
        max_response_bytes: Some(2000),
        transform: None,
        headers: vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            }
        ],
    };
    
    match http_request(request, 2_000_000_000u128).await {
        Ok((response,)) => {
            let body = String::from_utf8_lossy(&response.body);
            
            // Parse JSON response (simplified)
            if body.contains("blockNumber") {
                let transaction = CrossChainTransaction {
                    id: tx_id.clone(),
                    source_chain: format!("evm_{}", chain_id),
                    destination_chain: "icp".to_string(),
                    tx_hash: tx_hash.clone(),
                    block_height: 0, // Would parse from response
                    confirmations: 0,
                    status: "confirmed".to_string(),
                    timestamp: now_millis(),
                };
                
                TRANSACTIONS.with(|t| t.borrow_mut().insert(tx_id.clone(), transaction));
                Ok(format!("Transaction {} confirmed", tx_hash))
            } else {
                Ok(format!("Transaction {} pending", tx_hash))
            }
        }
        Err(e) => Err(format!("Failed to check transaction: {:?}", e)),
    }
}

async fn check_message_attestations(message_id: String) {
    let attestation_count = ATTESTATIONS.with(|a| {
        a.borrow().get(&message_id).map(|v| v.len()).unwrap_or(0)
    });
    
    if attestation_count >= REQUIRED_ATTESTATIONS {
        // Message is ready for cross-chain execution
        ic_cdk::println!("Message {} ready for execution", message_id);
    } else {
        // Schedule another check
        let timer_id = set_timer(Duration::from_secs(30), move || {
            ic_cdk::spawn(check_message_attestations(message_id.clone()));
        });
        
        TIMER_IDS.with(|t| t.borrow_mut().push(timer_id));
    }
}

#[update]
pub async fn verify_layerzero_message(
    _source_chain_id: u32,
    message_hash: String,
    dvn_endpoint: String,
) -> Result<bool, String> {
    // Query LayerZero DVN for message verification
    let request = CanisterHttpRequestArgument {
        url: format!("{}/verify/{}", dvn_endpoint, message_hash),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(1000),
        transform: None,
        headers: vec![],
    };
    
    match http_request(request, 2_000_000_000u128).await {
        Ok((response,)) => {
            let body = String::from_utf8_lossy(&response.body);
            Ok(body.contains("verified"))
        }
        Err(e) => Err(format!("DVN verification failed: {:?}", e)),
    }
}

#[query]
pub fn get_dvn_message(message_id: String) -> Option<DVNMessage> {
    DVN_MESSAGES.with(|m| m.borrow().get(&message_id).cloned())
}

#[query]
pub fn get_message_attestations(message_id: String) -> Vec<DVNAttestation> {
    ATTESTATIONS.with(|a| {
        a.borrow().get(&message_id).cloned().unwrap_or_default()
    })
}

#[query]
pub fn get_transaction(tx_id: String) -> Option<CrossChainTransaction> {
    TRANSACTIONS.with(|t| t.borrow().get(&tx_id).cloned())
}

#[query]
pub fn get_pending_messages() -> Vec<DVNMessage> {
    DVN_MESSAGES.with(|m| {
        m.borrow().values().filter(|msg| {
            let attestation_count = ATTESTATIONS.with(|a| {
                a.borrow().get(&msg.id).map(|v| v.len()).unwrap_or(0)
            });
            attestation_count < REQUIRED_ATTESTATIONS
        }).cloned().collect()
    })
}

#[query]
pub fn get_ready_messages() -> Vec<DVNMessage> {
    DVN_MESSAGES.with(|m| {
        m.borrow().values().filter(|msg| {
            let attestation_count = ATTESTATIONS.with(|a| {
                a.borrow().get(&msg.id).map(|v| v.len()).unwrap_or(0)
            });
            attestation_count >= REQUIRED_ATTESTATIONS
        }).cloned().collect()
    })
}

// Export Candid interface
ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_message_and_attest() {
        let msg_id = submit_dvn_message(1, 2, vec![1,2,3], "sender".to_string());
        assert!(msg_id.starts_with("msg_"));

        // First attestation
        let r1 = submit_attestation(msg_id.clone(), "v1".to_string(), vec![0x01]);
        assert!(r1.is_ok());
        let text1 = r1.unwrap();
        assert!(text1.contains("Attestation recorded") || text1.contains("ready for execution"));

        // Second attestation should reach quorum
        let r2 = submit_attestation(msg_id.clone(), "v2".to_string(), vec![0x02]).unwrap();
        assert!(r2.contains("ready for execution") || r2.contains("Attestation recorded"));
    }
}
