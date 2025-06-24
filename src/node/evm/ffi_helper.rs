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
use alloy_evm::precompiles::PrecompilesMap;
use alloy_primitives::{Bytes};
use alloy_rlp::Decodable;
use reth::revm::State;
use reth_evm_ethereum::RethReceiptBuilder;
use revm::inspector::NoOpInspector;
use crate::node::evm::BscEvm;
use crate::node::evm::executor::BscBlockExecutor;
use reth_primitives::TransactionSigned;
use reth_primitives_traits::SignerRecoverable;
use revm::Database;

/// 创建BscBlockExecutor的完整示例
pub fn create_bsc_block_executor<'a, DB: Database<Error: Send + Sync + 'static>>(
    db: &'a mut State<DB>,
    header: &'a Header,
) -> BscBlockExecutor<'a, BscEvm<&'a mut State<DB>, NoOpInspector, PrecompilesMap>, Arc<BscChainSpec>, RethReceiptBuilder> {
    // 1. 创建链规范
    let chain_spec = Arc::new(BscChainSpec { inner: bsc::bsc_mainnet() });

    // 2. 创建EVM配置
    let evm_config = BscEvmConfig::bsc(chain_spec.clone());

    // 3. 创建EVM实例
    let evm = BscEvmFactory::default().create_evm(db, evm_config.evm_env(header));

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
    let executor = BscBlockExecutor::new(
        evm,
        ctx,
        chain_spec,
        receipt_builder,
        system_contracts,
    );
    executor
}

pub fn batch_execute_transactions<'a, DB: Database<Error: Send + Sync + 'static>>(
    mut executor: BscBlockExecutor<'a, BscEvm<&'a mut State<DB>, NoOpInspector, PrecompilesMap>, Arc<BscChainSpec>, RethReceiptBuilder>,
    transactions : Vec<String>,
) {

    for tx_bytes in &transactions {
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
            Ok(gas_used) => println!("交易执行成功，消耗gas: {}", gas_used),
            Err(e) => println!("交易执行失败: {:?}", e),
        }
    }
}

pub fn creat_block_executor_and_run<DB: Database<Error: Send + Sync + 'static>>(
    db: &mut State<DB>,
    header: &Header,
    transactions : Vec<String>,
) {
    let executor = create_bsc_block_executor(db, header);
    batch_execute_transactions(executor, transactions);
}

/// 创建BscBlockExecutor的完整示例
pub fn create_bsc_block_executor_abc<DB: Database<Error: Send + Sync + 'static>>(
    db: &mut State<DB>,
    header: & Header,
    transactions : Vec<String>,
) {
    // 1. 创建链规范
    let chain_spec = Arc::new(BscChainSpec { inner: bsc::bsc_mainnet() });

    // 2. 创建EVM配置
    let evm_config = BscEvmConfig::bsc(chain_spec.clone());

    // 3. 创建EVM实例
    let evm = BscEvmFactory::default().create_evm(db, evm_config.evm_env(header));

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

    for tx_bytes in &transactions {
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
            Ok(gas_used) => println!("交易执行成功，消耗gas: {}", gas_used),
            Err(e) => println!("交易执行失败: {:?}", e),
        }
    }
}

mod tests {
    use super::*;
    use alloy_consensus::{TxLegacy};
    use alloy_primitives::{hex, TxKind, U256};
    use alloy_rlp::Encodable;
    use reth::revm::db::StateBuilder;
    use reth_chainspec::EthChainSpec;
    use revm::database::InMemoryDB;

    #[test]
    fn test_create_bsc_block_executor() {
        let header = Header::default();

        let empty_db = InMemoryDB::default();
        let mut db = StateBuilder::new_with_database(empty_db).build();
        let executor = create_bsc_block_executor(&mut db, &header);
        let tx = TxLegacy{
            chain_id: Option::from(bsc::bsc_mainnet().chain_id()),
            nonce: 0,
            gas_price: 0,
            gas_limit: 3000000,
            to: TxKind::default(),
            value: U256::try_from(0).unwrap(),
            input: Bytes::default(),
        };
        let mut buf: Vec<u8> = Vec::new();
        tx.encode(&mut buf);
        let hex_str = hex::encode(&buf);
        batch_execute_transactions(executor, vec![hex_str]);
    }

    #[test]
    fn test_create_bsc_block_executor_and_run() {
        let header = Header::default();

        let empty_db = InMemoryDB::default();
        let mut db = StateBuilder::new_with_database(empty_db).build();
        let tx = TxLegacy{
            chain_id: Option::from(bsc::bsc_mainnet().chain_id()),
            nonce: 0,
            gas_price: 0,
            gas_limit: 3000000,
            to: TxKind::default(),
            value: U256::try_from(0).unwrap(),
            input: Bytes::default(),
        };
        let mut buf = Vec::new();
        tx.encode(&mut buf);
        let hex_str = hex::encode(&buf);
        creat_block_executor_and_run(&mut db, &header, vec![hex_str.clone()]);

        create_bsc_block_executor_abc(&mut db, &header, vec![hex_str]);
    }
}