// use std::collections::HashMap;
// use std::error::Error;
// use std::fmt;
// use revm::{
//     database::{Database, DatabaseCommit, DBErrorMarker},
//     primitives::{Address, B256, StorageKey, StorageValue, U256},
//     state::{Account, AccountInfo, Bytecode},
// };
// use alloy_primitives::map::foldhash::fast::RandomState;
//
// /// 自定义数据库错误类型
// #[derive(Debug, Clone)]
// pub struct CustomDBError(String);
//
// impl fmt::Display for CustomDBError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "CustomDBError: {}", self.0)
//     }
// }
//
// impl Error for CustomDBError {}
//
// impl DBErrorMarker for CustomDBError {}
//
// /// 自定义数据库实现
// ///
// /// 这是一个简单的内存数据库实现，用于演示如何实现 Database 和 DatabaseCommit trait
// #[derive(Debug, Clone)]
// pub struct CustomDB {
//     /// 账户信息存储
//     accounts: HashMap<Address, AccountInfo>,
//     /// 账户代码存储
//     codes: HashMap<B256, Bytecode>,
//     /// 存储数据
//     storage: HashMap<Address, HashMap<StorageKey, StorageValue>>,
//     /// 区块哈希存储
//     block_hashes: HashMap<u64, B256>,
// }
//
// impl CustomDB {
//     /// 创建新的自定义数据库实例
//     pub fn new() -> Self {
//         Self {
//             accounts: HashMap::new(),
//             codes: HashMap::new(),
//             storage: HashMap::new(),
//             block_hashes: HashMap::new(),
//         }
//     }
//
//     /// 设置账户信息
//     pub fn set_account(&mut self, address: Address, account: AccountInfo) {
//         self.accounts.insert(address, account);
//     }
//
//     /// 设置账户代码
//     pub fn set_code(&mut self, code_hash: B256, code: Bytecode) {
//         self.codes.insert(code_hash, code);
//     }
//
//     /// 设置存储值
//     pub fn set_storage(&mut self, address: Address, key: StorageKey, value: StorageValue) {
//         self.storage
//             .entry(address)
//             .or_insert_with(HashMap::new)
//             .insert(key, value);
//     }
//
//     /// 设置区块哈希
//     pub fn set_block_hash(&mut self, number: u64, hash: B256) {
//         self.block_hashes.insert(number, hash);
//     }
//
//     /// 获取账户数量
//     pub fn account_count(&self) -> usize {
//         self.accounts.len()
//     }
//
//     /// 获取代码数量
//     pub fn code_count(&self) -> usize {
//         self.codes.len()
//     }
//
//     /// 获取存储条目数量
//     pub fn storage_count(&self) -> usize {
//         self.storage.values().map(|m| m.len()).sum()
//     }
// }
//
// impl Default for CustomDB {
//     fn default() -> Self {
//         Self::new()
//     }
// }
//
// /// 实现 Database trait
// impl Database for CustomDB {
//     type Error = CustomDBError;
//
//     /// 获取账户基本信息
//     fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
//         Ok(self.accounts.get(&address).cloned())
//     }
//
//     /// 根据代码哈希获取代码
//     fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
//         self.codes
//             .get(&code_hash)
//             .cloned()
//             .ok_or_else(|| CustomDBError(format!("Code not found for hash: {:?}", code_hash)))
//     }
//
//     /// 获取存储值
//     fn storage(&mut self, address: Address, index: StorageKey) -> Result<StorageValue, Self::Error> {
//         let value = self
//             .storage
//             .get(&address)
//             .and_then(|storage| storage.get(&index))
//             .copied()
//             .unwrap_or(StorageValue::default());
//         Ok(value)
//     }
//
//     /// 获取区块哈希
//     fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
//         self.block_hashes
//             .get(&number)
//             .copied()
//             .ok_or_else(|| CustomDBError(format!("Block hash not found for number: {}", number)))
//     }
// }
//
// /// 实现 DatabaseCommit trait
// impl DatabaseCommit for CustomDB {
//     /// 提交更改到数据库
//     fn commit(&mut self, changes: HashMap<Address, Account, RandomState>) {
//         println!("[CustomDB] Committing {} account changes", changes.len());
//
//         for (address, account) in changes {
//             // 更新账户信息
//             if let Some(info) = account.info {
//                 self.accounts.insert(address, info);
//                 println!("[CustomDB] Updated account: 0x{:x}", address);
//             }
//
//             // 更新存储
//             for (key, value) in account.changed_storage_slots() {
//                 self.set_storage(address, *key, *value);
//                 println!(
//                     "[CustomDB] Updated storage: addr=0x{:x}, key={:#x}, value={:#x}",
//                     address, key, value.present_value()
//                 );
//             }
//         }
//     }
// }
//
// /// 实现 DatabaseRef trait（可选，用于只读访问）
// impl revm::database::DatabaseRef for CustomDB {
//     type Error = CustomDBError;
//
//     fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
//         Ok(self.accounts.get(&address).cloned())
//     }
//
//     fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
//         self.codes
//             .get(&code_hash)
//             .cloned()
//             .ok_or_else(|| CustomDBError(format!("Code not found for hash: {:?}", code_hash)))
//     }
//
//     fn storage_ref(&self, address: Address, index: StorageKey) -> Result<StorageValue, Self::Error> {
//         let value = self
//             .storage
//             .get(&address)
//             .and_then(|storage| storage.get(&index))
//             .copied()
//             .unwrap_or(StorageValue::default());
//         Ok(value)
//     }
//
//     fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
//         self.block_hashes
//             .get(&number)
//             .copied()
//             .ok_or_else(|| CustomDBError(format!("Block hash not found for number: {}", number)))
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use revm::primitives::{address, b256};
//
//     #[test]
//     fn test_custom_db_basic_operations() {
//         let mut db = CustomDB::new();
//
//         // 创建测试地址和账户信息
//         let test_address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
//         let account_info = AccountInfo {
//             balance: U256::from(1000000000000000000u64), // 1 ETH
//             nonce: 42,
//             code_hash: B256::ZERO,
//             code: None,
//         };
//
//         // 设置账户
//         db.set_account(test_address, account_info.clone());
//
//         // 测试获取账户信息
//         let retrieved = db.basic(test_address).unwrap().unwrap();
//         assert_eq!(retrieved.balance, account_info.balance);
//         assert_eq!(retrieved.nonce, account_info.nonce);
//
//         // 测试获取不存在的账户
//         let non_existent = address!("0000000000000000000000000000000000000001");
//         let result = db.basic(non_existent).unwrap();
//         assert!(result.is_none());
//     }
//
//     #[test]
//     fn test_custom_db_storage() {
//         let mut db = CustomDB::new();
//         let test_address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
//         let storage_key = U256::from(123);
//         let storage_value = StorageValue::new(U256::from(456));
//
//         // 设置存储
//         db.set_storage(test_address, storage_key, storage_value);
//
//         // 测试获取存储值
//         let retrieved = db.storage(test_address, storage_key).unwrap();
//         assert_eq!(retrieved.present_value(), U256::from(456));
//
//         // 测试获取不存在的存储值
//         let non_existent_key = U256::from(999);
//         let result = db.storage(test_address, non_existent_key).unwrap();
//         assert_eq!(result.present_value(), U256::ZERO);
//     }
//
//     #[test]
//     fn test_custom_db_code() {
//         let mut db = CustomDB::new();
//         let code_hash = b256!("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
//         let code = Bytecode::new_raw(vec![0x60, 0x00, 0x52, 0x60, 0x20, 0x52].into());
//
//         // 设置代码
//         db.set_code(code_hash, code.clone());
//
//         // 测试获取代码
//         let retrieved = db.code_by_hash(code_hash).unwrap();
//         assert_eq!(retrieved, code);
//
//         // 测试获取不存在的代码
//         let non_existent_hash = b256!("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
//         let result = db.code_by_hash(non_existent_hash);
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn test_custom_db_block_hash() {
//         let mut db = CustomDB::new();
//         let block_number = 12345;
//         let block_hash = b256!("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
//
//         // 设置区块哈希
//         db.set_block_hash(block_number, block_hash);
//
//         // 测试获取区块哈希
//         let retrieved = db.block_hash(block_number).unwrap();
//         assert_eq!(retrieved, block_hash);
//
//         // 测试获取不存在的区块哈希
//         let result = db.block_hash(99999);
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn test_custom_db_commit() {
//         let mut db = CustomDB::new();
//         let test_address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
//
//         // 创建账户变更
//         let mut changes = HashMap::new();
//
//         // 创建账户信息
//         let account_info = AccountInfo {
//             balance: U256::from(1000000000000000000u64),
//             nonce: 1,
//             code_hash: B256::ZERO,
//             code: None,
//         };
//
//         // 创建存储变更
//         let storage_key = U256::from(1);
//         let storage_value = StorageValue::new(U256::from(100));
//         let mut storage = HashMap::new();
//         storage.insert(storage_key, storage_value);
//
//         // 创建Account结构体
//         let account = Account {
//             info: Some(account_info),
//             storage,
//         };
//
//         changes.insert(test_address, account);
//
//         // 提交变更
//         db.commit(changes);
//
//         // 验证变更是否生效
//         let account_info = db.basic(test_address).unwrap().unwrap();
//         assert_eq!(account_info.balance, U256::from(1000000000000000000u64));
//         assert_eq!(account_info.nonce, 1);
//
//         let storage_value = db.storage(test_address, storage_key).unwrap();
//         assert_eq!(storage_value.present_value(), U256::from(100));
//     }
// }
