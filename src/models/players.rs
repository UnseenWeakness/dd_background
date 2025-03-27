use serde::{Deserialize, Serialize};
use rbatis::RBatis;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Players {
    pub id: Option<i32>,
    pub nickname: Option<String>,
    pub wins: Option<i32>,
}

rbatis::crud!(Players {});

pub async fn get_all(rb: &RBatis) -> Result<Vec<Players>, rbatis::Error> {
    Players::select_all(rb).await
}