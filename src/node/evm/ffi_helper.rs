use std::error::Error;
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
use revm::{database::StateBuilder, Database};
use std::sync::Arc;
use alloy_consensus::{EthereumTxEnvelope, Header};
use alloy_evm::block::BlockExecutor;
use alloy_evm::{EvmFactory};
use alloy_evm::precompiles::PrecompilesMap;
use alloy_primitives::{Bytes};
use alloy_rlp::Decodable;
use reth::revm::State;
use reth_evm_ethereum::RethReceiptBuilder;
use revm::database::DBErrorMarker;
use revm::inspector::NoOpInspector;
use crate::node::evm::BscEvm;
use crate::node::evm::executor::BscBlockExecutor;

/// 创建BscBlockExecutor的完整示例
pub fn create_bsc_block_executor(
    db: Box<dyn Database<Error=dyn Error + Send + Sync>>,
    header: &Header,
) ->  BscBlockExecutor<BscEvm<State<Box<dyn Database<Error=dyn Error+Send+Sync>>>, NoOpInspector, PrecompilesMap>, Arc<BscChainSpec>, RethReceiptBuilder> {
    // 1. 创建链规范
    let chain_spec = Arc::new(BscChainSpec { inner: bsc::bsc_mainnet() });

    // 2. 创建EVM配置
    let evm_config = BscEvmConfig::bsc(chain_spec.clone());

    // 3. 创建数据库状态
    let _db = StateBuilder::new_with_database(db).build();

    // 4. 创建EVM环境
    // let evm_env = evm_config.evm_env(header);

    // 5. 创建EVM实例
    let evm = BscEvmFactory::default().create_evm(_db, evm_config.evm_env(header));

    // 6. 创建执行上下文 todo: fix
    let ctx = EthBlockExecutionCtx {
        parent_hash: header.parent_hash,
        parent_beacon_block_root: None,
        ommers: &[],
        withdrawals: None,
    };

    // 7. 创建收据构建器
    let receipt_builder = RethReceiptBuilder::default();

    // 8. 创建系统合约
    let system_contracts = SystemContract::new(chain_spec.clone());

    // 9. 创建BscBlockExecutor
    let mut executor = BscBlockExecutor::new(
        evm,
        ctx,
        chain_spec,
        receipt_builder,
        system_contracts,
    );
    executor
    // executor.execute_transaction();
}

pub fn batch_execute_transactions(
    mut executor: BscBlockExecutor<BscEvm<State<Box<dyn Database<Error=dyn Error+Send+Sync>>>, NoOpInspector, PrecompilesMap>, Arc<BscChainSpec>, RethReceiptBuilder>,
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
        let tx_envelop = EthereumTxEnvelope::decode(&mut &_tx_bytes[..]).unwrap();
        // let transaction = TransactionSigned::decode(&mut &Bytes::from_str(tx_bytes)[..])?
        //     .try_into_recovered()
        //     .map_err(|tx| eyre::eyre!("failed to recover tx: {}", tx.tx_hash()))?;

        let result = executor.execute_transaction_with_result_closure(
            tx_envelop.into_typed_transaction(),
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

pub fn creat_block_executor_and_run(
    db: Box<dyn Database<Error=dyn Error>>,
    header: &Header,
    transactions : Vec<String>,
) {
    let executor = create_bsc_block_executor(db, header);
    batch_execute_transactions(executor, transactions);
}

mod tests {
    use alloy_consensus::{Header, TxLegacy};
    use alloy_rlp::Encodable;
    use reth_chainspec::EthChainSpec;
    use crate::chainspec::bsc::bsc_mainnet;
    use crate::node::evm::ffi_helper::{batch_execute_transactions, creat_block_executor_and_run};
    use crate::node::evm::ffi_helper::create_bsc_block_executor;

    #[test]
    fn test_create_bsc_block_executor() {
        let db = Box::new(revm::database::InMemoryDB::default());
        let header = Header::default();
        let executor = create_bsc_block_executor(db, &header);
        assert!(executor.is_ok());
        let tx = TxLegacy{
            chain_id: bsc_mainnet().chain_id(),
            nonce: 0,
            gas_price: 0,
            gas_limit: 3000000,
            to: None,
            value: 0,
            input: Vec::new(),
        };
        let mut buf = Vec::new();
        tx.encode(&mut buf);
        batch_execute_transactions(executor.unwrap(), vec![buf.to_string()]);
    }
}