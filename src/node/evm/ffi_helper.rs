use std::str::FromStr;
use crate::{
    chainspec::{bsc, BscChainSpec},
    node::evm::{
        config::BscEvmConfig,
        factory::BscEvmFactory,
    },
    system_contracts::SystemContract,
};
use reth_evm::{eth::EthBlockExecutionCtx, ConfigureEvm};
use std::sync::Arc;
use alloy_consensus::{EthereumTxEnvelope, Header, TxEip4844};
use alloy_evm::block::BlockExecutor;
use alloy_evm::{EvmFactory};
use alloy_primitives::{Bytes};
use alloy_rlp::{Decodable, encode};
use reth::revm::db::StateBuilder;
use reth_evm_ethereum::RethReceiptBuilder;
use crate::node::evm::executor::BscBlockExecutor;
use reth_primitives::TransactionSigned;
use reth_primitives_traits::SignerRecoverable;
use revm::Database;
use triehash::ordered_trie_root;
use keccak_hasher::KeccakHasher;
use std::ffi::CString;
use std::os::raw::c_char;
use std::time::{Duration, Instant, SystemTime};
use alloy_consensus::transaction::Recovered;
use hex;
use once_cell::unsync::Lazy;

static CHAIN_SPEC: Lazy<Arc<BscChainSpec> >= Lazy::new(||Arc::new(BscChainSpec { inner: bsc::bsc_mainnet() }));
static EVM_CONFIG: BscEvmConfig = BscEvmConfig::bsc(CHAIN_SPEC);
static RECEIPT_BUILDER: RethReceiptBuilder = RethReceiptBuilder::default();

// 6. 创建系统合约
// static SYSTEM_CONTRACTS: SystemContract<Arc<BscChainSpec>> = SystemContract::new(CHAIN_SPEC);

struct ABC {

}

/// 创建BscBlockExecutor的完整示例
pub fn batch_run_txs<DB: Database<Error: Send + Sync + 'static>>(
    db: DB,
    header: & Header,
    transactions : Vec<&str>,
) -> (u64, [u8;32], *mut c_char) {
    let mut rust_transactions: Vec<Recovered<TransactionSigned>> = vec![];
    for tx_bytes in transactions {
        let _tx_bytes = match Bytes::from_str(tx_bytes) {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("无法解析交易字节: {:?}", e);
                continue;
            }
        };

        // 解析交易
        let tx_envelop: alloy_consensus::EthereumTxEnvelope<TxEip4844>  = match EthereumTxEnvelope::decode(&mut &_tx_bytes[..]) {
            Ok(envelope) => envelope,
            Err(e) => {
                println!("无法解码交易: {:?}", e);
                continue;
            }
        };

        // 转换为TransactionSigned
        let tx: TransactionSigned = match tx_envelop.try_into() {
            Ok(tx) => tx,
            Err(e) => {
                println!("无法转换交易: {:?}", e);
                continue;
            }
        };

        // 恢复签名者
        let recovered = match tx.try_into_recovered() {
            Ok(recovered) => recovered,
            Err(e) => {
                println!("无法恢复签名者: {:?}", e);
                continue;
            }
        };
        rust_transactions.push(recovered);

    }

    let st = Instant::now();
    // let mut total_real = Duration::new(0, 0);
    // // 1. 创建链规范
    // let CHAIN_SPEC = Arc::new(BscChainSpec { inner: bsc::bsc_mainnet() });
    //
    // // 2. 创建EVM配置
    // let EVM_CONFIG = BscEvmConfig::bsc(CHAIN_SPEC.clone());

    let mut db = StateBuilder::new_with_database(db).build();
    // 3. 创建EVM实例
    let evm = BscEvmFactory::default().create_evm(&mut db, EVM_CONFIG.evm_env(header));

    // 4. 创建执行上下文
    let ctx = EthBlockExecutionCtx {
        parent_hash: header.parent_hash,
        parent_beacon_block_root: None,
        ommers: &[],
        withdrawals: None,
    };

    // 7. 创建BscBlockExecutor
    let mut executor = BscBlockExecutor::new(
        evm,
        ctx,
        CHAIN_SPEC.clone(),
        RECEIPT_BUILDER,
        SystemContract::new(CHAIN_SPEC.clone()),
    );

    let st_txs_time = Instant::now();
    // let mut total_gas: u64 = 0;


    // let pure_time = Instant::now();
    for recovered in rust_transactions {

        // let syst = SystemTime::now();
        let result = executor.execute_transaction_with_result_closure(
            &recovered,
            |result| {
                // println!("交易执行结果: {:?}", result);
                // total_real += syst.elapsed().unwrap();
            },
        );
        match result {
            Ok(gas_used) => {
                // println!("交易执行成功，消耗gas: {}", gas_used);
                // total_gas += gas_used;
            },
            Err(e) => println!("交易执行失败: {:?}", e),
        }
    }

    // println!("batch total gas_used: {}", total_gas);

    // Finish executor to get receipts and compute root
    let (_evm, exec_result) = executor.finish().expect("executor finish failed");

    println!("whole batch_run_txs time: {:?}, txs time: {:?}", st.elapsed(), st_txs_time.elapsed());
    // let receipts = exec_result.receipts;
    //
    // // 为计算 root，单独编码每个 receipt
    // use alloy_rlp::Encodable;
    // let encoded_receipts: Vec<Vec<u8>> = receipts
    //     .iter()
    //     .map(|r| {
    //         let mut buf = Vec::new();
    //         r.encode(&mut buf);
    //         buf
    //     })
    //     .collect();
    //
    // let root = ordered_trie_root::<KeccakHasher, _>(encoded_receipts.iter().map(|v| &v[..]));
    //
    // let receipts_bytes = encode(receipts);
    // let receipts_hex = hex::encode(receipts_bytes);
    let c_str = CString::new("aaa").unwrap();
    // let ptr = c_str.into_raw();

    let root: [u8;32] = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32];
    (exec_result.gas_used, root, c_str.into_raw())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::{TxLegacy};
    use alloy_primitives::{hex, TxKind, U256, Address, Signature};
    use alloy_rlp::Encodable;
    use reth_chainspec::EthChainSpec;
    use revm::database::InMemoryDB;
    use secp256k1::{SecretKey, Secp256k1};
    use reth_primitives::{Transaction};
    use std::str::FromStr;

    #[test]
    fn test_create_bsc_block_executor_and_run() {
        let mut header = Header::default();

        header.gas_limit = 30000000;

        // 创建交易
        let tx = TxLegacy{
            chain_id: Option::from(bsc::bsc_mainnet().chain_id()),
            nonce: 0,
            gas_price: u128::try_from(U256::from(20000000000u64)).unwrap(), // 20 Gwei
            gas_limit: 3000000,
            to: TxKind::Call(Address::from_str("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6").unwrap()), // 发送到另一个地址
            value: U256::try_from(0).unwrap(),
            input: Bytes::default(),
        };

        // 使用私钥签名交易
        let private_key_hex = "a18e014010947d1639014caac07283b664e776c9e4b5556ab2e8c6502359c287";
        let private_key_bytes = hex::decode(private_key_hex).unwrap();
        let secret_key = SecretKey::from_slice(&private_key_bytes).unwrap();
        
        // 创建签名上下文
        let secp = Secp256k1::new();
        
        // 编码交易用于签名
        let mut buf = Vec::new();
        tx.encode(&mut buf);
        
        // 计算交易哈希
        let tx_hash = alloy_primitives::keccak256(&buf);
        
        // 签名交易哈希
        let message = secp256k1::Message::from_digest_slice(tx_hash.as_slice()).unwrap();
        let signature = secp.sign_ecdsa(&message, &secret_key);
        
        // 创建签名对象
        let signature_obj = Signature::new(
            U256::from_be_slice(&signature.serialize_compact()[0..32]),
            U256::from_be_slice(&signature.serialize_compact()[32..64]),
            false,
        );
        
        // 创建签名后的交易
        let signed_tx = TransactionSigned::new_unhashed(
            Transaction::Legacy(tx),
            signature_obj,
        );
        
        // 编码签名后的交易
        let mut signed_buf = Vec::new();
        signed_tx.encode(&mut signed_buf);
        let hex_str = hex::encode(&signed_buf);

        println!("签名后的交易hex: {}", hex_str);
        println!("发送者地址: 0xe4661eb2c39422d41056bdf629b446a2e514e9fc");
        println!("接收者地址: 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6");

        batch_run_txs(&mut InMemoryDB::default(), &header, vec![&*hex_str]);
    }

}