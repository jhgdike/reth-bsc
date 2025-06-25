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
use alloy_rlp::Decodable;
use reth::revm::db::StateBuilder;
use reth_evm_ethereum::RethReceiptBuilder;
use crate::node::evm::executor::BscBlockExecutor;
use reth_primitives::TransactionSigned;
use reth_primitives_traits::SignerRecoverable;
use revm::Database;

/// 创建BscBlockExecutor的完整示例
pub fn batch_run_txs<DB: Database<Error: Send + Sync + 'static>>(
    db: DB,
    header: & Header,
    transactions : Vec<&str>,
) -> u64 {
    // 1. 创建链规范
    let chain_spec = Arc::new(BscChainSpec { inner: bsc::bsc_mainnet() });

    // 2. 创建EVM配置
    let evm_config = BscEvmConfig::bsc(chain_spec.clone());

    let mut db = StateBuilder::new_with_database(db).build();
    // 3. 创建EVM实例
    let evm = BscEvmFactory::default().create_evm(&mut db, evm_config.evm_env(header));

    // 4. 创建执行上下文
    let ctx = EthBlockExecutionCtx {
        parent_hash: header.parent_hash,
        parent_beacon_block_root: None,
        ommers: &[],
        withdrawals: None,
    };

    // 5. 创建收据构建器
    let receipt_builder = RethReceiptBuilder::default();

    // 6. 创建系统合约
    let system_contracts = SystemContract::new(chain_spec.clone());

    // 7. 创建BscBlockExecutor
    let mut executor = BscBlockExecutor::new(
        evm,
        ctx,
        chain_spec,
        receipt_builder,
        system_contracts,
    );

    let mut total_gas: u64 = 0;
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

        let result = executor.execute_transaction_with_result_closure(
            &recovered,
            |result| {
                println!("交易执行结果: {:?}", result);
            },
        );
        match result {
            Ok(gas_used) => {
                println!("交易执行成功，消耗gas: {}", gas_used);
                total_gas += gas_used;
            },
            Err(e) => println!("交易执行失败: {:?}", e),
        }
    }

    println!("batch total gas_used: {}", total_gas);
    total_gas
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