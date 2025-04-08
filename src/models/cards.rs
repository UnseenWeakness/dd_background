use std::sync::Arc;
use serde::{Deserialize, Serialize};
use rbatis::RBatis;
use moka::future::Cache;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub attack: Option<i32>,
    pub health: Option<i32>,
    pub rarity: Option<String>,
}

// 指定cards 表名
rbatis::crud!(Card {}, "cards");

pub async fn get_all(rb: &RBatis, cache: Arc<Cache<String, Vec<Card>>>) -> Result<Vec<Card>, rbatis::Error> {
    let cache_key = "all_cards".to_string();
    cache
        .try_get_with(cache_key.clone(), async {
            log::info!("Cache miss, querying database");
            Card::select_all(rb).await // 直接返回 Result<Vec<Card>, rbatis::Error>
        })
        .await
        .map_err(|e| rbatis::Error::from(e.to_string())) // 将 Arc<dyn Error> 转为 rbatis::Error
    // Card::select_all(rb).await
}