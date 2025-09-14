use candid::{CandidType, Deserialize};
use ic_cdk::{query, update, api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
}};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Clone)]
pub struct EVMChainConfig {
    pub chain_id: u32,
    pub name: String,
    pub rpc_url: String,
    pub block_explorer: String,
    pub native_token: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TransactionReceipt {
    pub tx_hash: String,
    pub block_number: u64,
    pub block_hash: String,
    pub transaction_index: u32,
    pub from_address: String,
    pub to_address: String,
    pub gas_used: u64,
    pub status: bool,
    pub logs: Vec<EVMLog>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct EVMLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub log_index: u32,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub transaction_count: u32,
}

thread_local! {
    static CHAIN_CONFIGS: std::cell::RefCell<HashMap<u32, EVMChainConfig>> = std::cell::RefCell::new(HashMap::new());
    static CACHED_RECEIPTS: std::cell::RefCell<HashMap<String, TransactionReceipt>> = std::cell::RefCell::new(HashMap::new());
    static CACHED_BLOCKS: std::cell::RefCell<HashMap<u64, BlockInfo>> = std::cell::RefCell::new(HashMap::new());
}

#[update]
pub fn init_chain_configs() {
    let configs = vec![
        EVMChainConfig {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://eth-mainnet.g.alchemy.com/v2/demo".to_string(),
            block_explorer: "https://etherscan.io".to_string(),
            native_token: "ETH".to_string(),
        },
        EVMChainConfig {
            chain_id: 11155111,
            name: "Sepolia Testnet".to_string(),
            rpc_url: "https://eth-sepolia.g.alchemy.com/v2/demo".to_string(),
            block_explorer: "https://sepolia.etherscan.io".to_string(),
            native_token: "ETH".to_string(),
        },
        EVMChainConfig {
            chain_id: 137,
            name: "Polygon Mainnet".to_string(),
            rpc_url: "https://polygon-rpc.com".to_string(),
            block_explorer: "https://polygonscan.com".to_string(),
            native_token: "MATIC".to_string(),
        },
        EVMChainConfig {
            chain_id: 80001,
            name: "Mumbai Testnet".to_string(),
            rpc_url: "https://rpc-mumbai.maticvigil.com".to_string(),
            block_explorer: "https://mumbai.polygonscan.com".to_string(),
            native_token: "MATIC".to_string(),
        },
    ];
    
    CHAIN_CONFIGS.with(|c| {
        let mut chains = c.borrow_mut();
        for config in configs {
            chains.insert(config.chain_id, config);
        }
    });
}

#[update]
pub async fn get_transaction_receipt(
    chain_id: u32,
    tx_hash: String,
) -> Result<TransactionReceipt, String> {
    // Check cache first
    if let Some(receipt) = CACHED_RECEIPTS.with(|c| c.borrow().get(&tx_hash).cloned()) {
        return Ok(receipt);
    }
    
    let rpc_url = CHAIN_CONFIGS.with(|c| {
        c.borrow().get(&chain_id).map(|config| config.rpc_url.clone())
    }).ok_or("Chain not configured")?;
    
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "eth_getTransactionReceipt",
        "params": [tx_hash],
        "id": 1
    }).to_string();
    
    let request = CanisterHttpRequestArgument {
        url: rpc_url,
        method: HttpMethod::POST,
        body: Some(request_body.into_bytes()),
        max_response_bytes: Some(5000),
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
            parse_transaction_receipt(&body, &tx_hash)
        }
        Err(e) => Err(format!("RPC request failed: {:?}", e)),
    }
}

#[update]
pub async fn get_block_info(
    chain_id: u32,
    block_number: u64,
) -> Result<BlockInfo, String> {
    // Check cache first
    if let Some(block) = CACHED_BLOCKS.with(|c| c.borrow().get(&block_number).cloned()) {
        return Ok(block);
    }
    
    let rpc_url = CHAIN_CONFIGS.with(|c| {
        c.borrow().get(&chain_id).map(|config| config.rpc_url.clone())
    }).ok_or("Chain not configured")?;
    
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "eth_getBlockByNumber",
        "params": [format!("0x{:x}", block_number), false],
        "id": 1
    }).to_string();
    
    let request = CanisterHttpRequestArgument {
        url: rpc_url,
        method: HttpMethod::POST,
        body: Some(request_body.into_bytes()),
        max_response_bytes: Some(3000),
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
            parse_block_info(&body, block_number)
        }
        Err(e) => Err(format!("RPC request failed: {:?}", e)),
    }
}

#[update]
pub async fn get_latest_block_number(chain_id: u32) -> Result<u64, String> {
    let rpc_url = CHAIN_CONFIGS.with(|c| {
        c.borrow().get(&chain_id).map(|config| config.rpc_url.clone())
    }).ok_or("Chain not configured")?;
    
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    }).to_string();
    
    let request = CanisterHttpRequestArgument {
        url: rpc_url,
        method: HttpMethod::POST,
        body: Some(request_body.into_bytes()),
        max_response_bytes: Some(1000),
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
            parse_block_number(&body)
        }
        Err(e) => Err(format!("RPC request failed: {:?}", e)),
    }
}

fn parse_transaction_receipt(json_str: &str, tx_hash: &str) -> Result<TransactionReceipt, String> {
    let v: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON parse error: {}", e))?;
    
    let result = v.get("result")
        .ok_or("No result in response")?;
    
    if result.is_null() {
        return Err("Transaction not found".to_string());
    }
    
    let receipt = TransactionReceipt {
        tx_hash: tx_hash.to_string(),
        block_number: parse_hex_to_u64(result.get("blockNumber").and_then(|v| v.as_str()).unwrap_or("0x0"))?,
        block_hash: result.get("blockHash").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        transaction_index: parse_hex_to_u32(result.get("transactionIndex").and_then(|v| v.as_str()).unwrap_or("0x0"))?,
        from_address: result.get("from").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        to_address: result.get("to").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        gas_used: parse_hex_to_u64(result.get("gasUsed").and_then(|v| v.as_str()).unwrap_or("0x0"))?,
        status: result.get("status").and_then(|v| v.as_str()).unwrap_or("0x0") == "0x1",
        logs: parse_logs(result.get("logs").unwrap_or(&Value::Array(vec![]))),
    };
    
    // Cache the receipt
    CACHED_RECEIPTS.with(|c| c.borrow_mut().insert(tx_hash.to_string(), receipt.clone()));
    
    Ok(receipt)
}

fn parse_block_info(json_str: &str, block_number: u64) -> Result<BlockInfo, String> {
    let v: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON parse error: {}", e))?;
    
    let result = v.get("result")
        .ok_or("No result in response")?;
    
    let block = BlockInfo {
        number: block_number,
        hash: result.get("hash").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        parent_hash: result.get("parentHash").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        timestamp: parse_hex_to_u64(result.get("timestamp").and_then(|v| v.as_str()).unwrap_or("0x0"))?,
        gas_limit: parse_hex_to_u64(result.get("gasLimit").and_then(|v| v.as_str()).unwrap_or("0x0"))?,
        gas_used: parse_hex_to_u64(result.get("gasUsed").and_then(|v| v.as_str()).unwrap_or("0x0"))?,
        transaction_count: result.get("transactions").and_then(|v| v.as_array()).map(|a| a.len() as u32).unwrap_or(0),
    };
    
    // Cache the block
    CACHED_BLOCKS.with(|c| c.borrow_mut().insert(block_number, block.clone()));
    
    Ok(block)
}

fn parse_block_number(json_str: &str) -> Result<u64, String> {
    let v: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON parse error: {}", e))?;
    
    let result = v.get("result")
        .and_then(|v| v.as_str())
        .ok_or("No result in response")?;
    
    parse_hex_to_u64(result)
}

fn parse_logs(logs_value: &Value) -> Vec<EVMLog> {
    logs_value.as_array().map(|logs| {
        logs.iter().enumerate().filter_map(|(i, log)| {
            Some(EVMLog {
                address: log.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                topics: log.get("topics").and_then(|v| v.as_array()).map(|topics| {
                    topics.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect()
                }).unwrap_or_default(),
                data: log.get("data").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                log_index: i as u32,
            })
        }).collect()
    }).unwrap_or_default()
}

fn parse_hex_to_u64(hex_str: &str) -> Result<u64, String> {
    let clean_hex = hex_str.trim_start_matches("0x");
    u64::from_str_radix(clean_hex, 16)
        .map_err(|e| format!("Failed to parse hex: {}", e))
}

fn parse_hex_to_u32(hex_str: &str) -> Result<u32, String> {
    let clean_hex = hex_str.trim_start_matches("0x");
    u32::from_str_radix(clean_hex, 16)
        .map_err(|e| format!("Failed to parse hex: {}", e))
}

#[query]
pub fn get_chain_config(chain_id: u32) -> Option<EVMChainConfig> {
    CHAIN_CONFIGS.with(|c| c.borrow().get(&chain_id).cloned())
}

#[query]
pub fn get_supported_chains() -> Vec<EVMChainConfig> {
    CHAIN_CONFIGS.with(|c| c.borrow().values().cloned().collect())
}

#[query]
pub fn get_cached_receipt(tx_hash: String) -> Option<TransactionReceipt> {
    CACHED_RECEIPTS.with(|c| c.borrow().get(&tx_hash).cloned())
}

#[query]
pub fn get_cached_block(block_number: u64) -> Option<BlockInfo> {
    CACHED_BLOCKS.with(|c| c.borrow().get(&block_number).cloned())
}

// Export Candid interface
ic_cdk::export_candid!();
