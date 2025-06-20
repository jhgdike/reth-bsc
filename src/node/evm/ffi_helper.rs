use crate::{
    chainspec::{bsc, BscChainSpec},
    node::evm::{
        config::BscEvmConfig,
        factory::BscEvmFactory,
        BscEvm,
    },
    system_contracts::SystemContract,
};
use reth_evm::{eth::EthBlockExecutionCtx, ConfigureEvm};
use reth_revm::State;
use revm::{inspector::NoOpInspector, database::StateBuilder, Database};
use std::sync::Arc;
use revm::context::DBErrorMarker;
use alloy_consensus::Header;
use alloy_evm::block::BlockExecutor;
use alloy_evm::EvmFactory;
use alloy_evm::precompiles::PrecompilesMap;
use reth_evm_ethereum::RethReceiptBuilder;
use crate::node::evm::executor::BscBlockExecutor;

/// 创建BscBlockExecutor的完整示例
// pub fn create_bsc_block_executor_example(db: Box<dyn Database<Error=dyn DBErrorMarker>>, header: &Header) -> BscBlockExecutor<BscEvm<State<Box<dyn Database<Error=dyn DBErrorMarker>>>, NoOpInspector, PrecompilesMap>, Arc<BscChainSpec>, RethReceiptBuilder> {
pub fn create_bsc_block_executor_example(db: Box<dyn Database<Error=dyn DBErrorMarker>>, header: &Header) {
    // 1. 创建链规范
    let chain_spec = Arc::new(BscChainSpec { inner: bsc::bsc_mainnet() });

    // 2. 创建EVM配置
    let evm_config = BscEvmConfig::bsc(chain_spec.clone());

    // 3. 创建数据库状态
    let db = StateBuilder::new_with_database(db).build();

    // 4. 创建EVM环境
    // let evm_env = evm_config.evm_env(header);

    // 5. 创建EVM实例
    let evm = BscEvmFactory::default().create_evm(db, evm_config.evm_env(header));

    // 6. 创建执行上下文
    let ctx = EthBlockExecutionCtx {
        parent_hash: alloy_primitives::B256::default(),
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
    executor.execute_transaction();
    // executor
}
