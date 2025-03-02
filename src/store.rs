use crate::utils::{calculate_block_hash, get_current_timestamp};

use rocksdb::{DB, IteratorMode, Direction};
use serde::{Serialize, Deserialize};
use bincode;

use std::convert::TryInto;

/// 事务（fine）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Transaction {
    Tx0 = 0,
    Tx1 = 1,
    Tx2 = 2,
}

/// 区块（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: Vec<u8>,
    pub hash: Vec<u8>,
}


/// `BlockStore` trait 定义数据库操作接口
pub trait BlockStore {
    fn put_block(&self, block: &Block) -> Result<(), String>;
    fn get_block_by_index(&self, index: u64) -> Result<Option<Block>, String>;
    fn get_last_block(&self) -> Result<Option<Block>, String>;
    fn get_blocks_in_range(&self, begin_index: u64, end_index: u64) -> Result<Option<Vec<Block>>, String>;
    fn create_block(&self, operations: &Vec<Transaction>) -> Result<Block, String>;
}


/// 使用 RocksDB 实现 `BlockStore` trait
pub struct RocksDBBlockStore {
    db: DB,
}

impl RocksDBBlockStore {
    pub fn new(path: &str) -> Result<Self, String> {
        let db = DB::open_default(path)?;
        
        let last_block_index_key = b"last_block_index";
        if let None = db.get(last_block_index_key)? {
            let genesis_block = Block {
                index: 0,
                timestamp: get_current_timestamp().unwrap(),
                transactions: Vec::new(),
                previous_hash: Vec::new(),
                hash: Vec::new(),
            };
            let mut batch = rocksdb::WriteBatch::default();
            batch.put(genesis_block.index.to_le_bytes(), bincode::serialize(&genesis_block)
                .map_err(|e| e.to_string())?);
            batch.put(last_block_index_key, genesis_block.index.to_le_bytes());
            db.write(batch)?;
        }
        Ok(Self{db})
    }
}

impl BlockStore for RocksDBBlockStore {
    fn put_block(&self, block: &Block) -> Result<(), String> {
        match self.get_last_block()? {
            Some(last_block) => {
                if last_block.index + 1 == block.index && last_block.hash == block.previous_hash {
                    let mut batch = rocksdb::WriteBatch::default();
                    batch.put(block.index.to_le_bytes(), bincode::serialize(block)
                        .map_err(|e| e.to_string())?);
                    batch.put(b"last_block_index", block.index.to_le_bytes());
                    self.db.write(batch)?;
                }
                Ok(())
            },
            None => {
                Err("缺失创世区块".to_string())
            }
        }
    }

    fn get_block_by_index(&self, index: u64) -> Result<Option<Block>, String> {
        let index = index.to_le_bytes();
        if let Some(value) = self.db.get(index)? {
            let block: Block = bincode::deserialize(&value)
                .map_err(|e| e.to_string())?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    fn get_last_block(&self) -> Result<Option<Block>, String> {
        let last_block_index_key = b"last_block_index";
        if let Some(last_index_bytes) = self.db.get(last_block_index_key)? {
            let last_index: u64 = u64::from_le_bytes(last_index_bytes.try_into().unwrap());
            self.get_block_by_index(last_index)
        } else {
            Ok(None)
        }
    }

    fn get_blocks_in_range(&self, begin_index: u64, end_index: u64) -> Result<Option<Vec<Block>>, String> {
        let mut iter = 
            self.db.iterator(IteratorMode::From(&begin_index.to_le_bytes(), Direction::Forward));
        let mut blocks = Vec::new();
        while let Some(Ok((key, value))) = iter.next() {
            // 跳过长度不为8的键（非区块索引）
            if key.len() != 8 {
                continue;
            }
            // 安全转换为u64
            let index_bytes: [u8; 8] = key.as_ref().try_into()
                .map_err(|e: std::array::TryFromSliceError| e.to_string())?;
            let index = u64::from_le_bytes(index_bytes);
            if index > end_index {
                break;
            }
            let block = bincode::deserialize(&value)
                .map_err(|e| e.to_string())?;
            blocks.push(block);
        }
        Ok(Some(blocks))
    }

    fn create_block(&self, transactions: &Vec<Transaction>) -> Result<Block, String> {
        if let Some(last_block) = self.get_last_block()? {
            // 生成新区块
            let index = last_block.index + 1;
            let timestamp = get_current_timestamp().unwrap();
            let previous_hash = last_block.hash;
            let hash = calculate_block_hash(index, timestamp, transactions, &previous_hash).unwrap();
        
            let new_block = Block {
                index,
                timestamp,
                transactions: transactions.clone(),
                previous_hash,
                hash,
            };
        
            Ok(new_block)
        } else {
            Err("缺失创世区块".to_string())
        }
    }
}

