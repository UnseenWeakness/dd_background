use serde::{Deserialize, Serialize};
use rbatis::RBatis;


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

pub async fn get_all(rb: &RBatis) -> Result<Vec<Card>, rbatis::Error> {
    Card::select_all(rb).await
}