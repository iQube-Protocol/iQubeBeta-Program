use candid::{CandidType, Deserialize};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
};
use ic_cdk::{query, update};
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Clone)]
pub struct Receipt {
    pub id: String,
    pub data_hash: String,
    pub timestamp: u64,
    pub merkle_proof: Vec<String>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct MerkleBatch {
    pub root: String,
    pub receipts: Vec<Receipt>,
    pub created_at: u64,
    pub btc_anchor_txid: Option<String>,
    pub btc_block_height: Option<u64>,
}

thread_local! {
    static RECEIPTS: std::cell::RefCell<HashMap<String, Receipt>> = std::cell::RefCell::new(HashMap::new());
    static BATCHES: std::cell::RefCell<Vec<MerkleBatch>> = std::cell::RefCell::new(Vec::new());
    static PENDING_RECEIPTS: std::cell::RefCell<Vec<Receipt>> = std::cell::RefCell::new(Vec::new());
}

#[update]
pub fn issue_receipt(data_hash: String) -> String {
    let receipt_id = format!("receipt_{}", ic_cdk::api::time());
    let receipt = Receipt {
        id: receipt_id.clone(),
        data_hash,
        timestamp: ic_cdk::api::time(),
        merkle_proof: vec![],
    };
    
    RECEIPTS.with(|r| r.borrow_mut().insert(receipt_id.clone(), receipt.clone()));
    PENDING_RECEIPTS.with(|p| p.borrow_mut().push(receipt));
    
    receipt_id
}

#[update]
pub fn batch() -> String {
    let pending = PENDING_RECEIPTS.with(|p| {
        let mut pending = p.borrow_mut();
        let batch = pending.clone();
        pending.clear();
        batch
    });
    
    if pending.is_empty() {
        return "No pending receipts".to_string();
    }
    
    // Build Merkle tree (simplified - just hash all receipt IDs)
    let mut hasher = Sha256::new();
    for receipt in &pending {
        hasher.update(receipt.id.as_bytes());
    }
    let root = format!("{:x}", hasher.finalize());
    
    let batch = MerkleBatch {
        root: root.clone(),
        receipts: pending,
        created_at: ic_cdk::api::time(),
        btc_anchor_txid: None,
        btc_block_height: None,
    };
    
    BATCHES.with(|b| b.borrow_mut().push(batch));
    
    root
}

#[update]
pub async fn anchor() -> String {
    let latest_batch = BATCHES.with(|b| {
        let batches = b.borrow();
        batches.last().cloned()
    });
    
    match latest_batch {
        Some(mut batch) => {
            // TODO: Call BTC signer canister to create anchor transaction
            // For now, return mock response
            batch.btc_anchor_txid = Some("mock_txid_123".to_string());
            batch.btc_block_height = Some(800000);
            
            BATCHES.with(|b| {
                let mut batches = b.borrow_mut();
                if let Some(last) = batches.last_mut() {
                    last.btc_anchor_txid = batch.btc_anchor_txid.clone();
                    last.btc_block_height = batch.btc_block_height;
                }
            });
            
            format!("Anchored batch {} to BTC", batch.root)
        }
        None => "No batches to anchor".to_string(),
    }
}

#[query]
pub fn get_receipt(receipt_id: String) -> Option<Receipt> {
    RECEIPTS.with(|r| r.borrow().get(&receipt_id).cloned())
}

#[query]
pub fn get_batches() -> Vec<MerkleBatch> {
    BATCHES.with(|b| b.borrow().clone())
}

#[query]
pub fn get_pending_count() -> usize {
    PENDING_RECEIPTS.with(|p| p.borrow().len())
}

// Export Candid interface
ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_and_count_pending() {
        let before = get_pending_count();
        let id = issue_receipt("deadbeef".to_string());
        assert!(id.starts_with("receipt_"));
        let after = get_pending_count();
        assert_eq!(after, before + 1);
    }

    #[test]
    fn batch_and_anchor_mock() {
        // Ensure at least one receipt exists
        let _ = issue_receipt("cafebabe".to_string());
        let root = batch();
        assert!(!root.is_empty());
        let batches = get_batches();
        assert!(!batches.is_empty());
        // anchor() is async but uses mock values; should succeed
        let res = futures::executor::block_on(anchor());
        assert!(res.contains("Anchored batch") || res == "No batches to anchor");
    }
}
