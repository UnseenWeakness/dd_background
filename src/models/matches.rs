use rbatis::RBatis;
use serde::{Deserialize, Serialize};

/// 对战表
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Match {
    pub id: Option<i32>,
    pub player1_id: Option<i32>,
    pub player2_id: Option<i32>,
    pub winner_id: Option<i32>,
    pub status: Option<String>,
    pub start_time: Option<rbatis::rbdc::datetime::DateTime>,
}

rbatis::crud!(Match {}, "matches");

/// 对战中的卡牌状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchCard {
    pub match_id: Option<i32>,
    pub player_id: Option<i32>,
    pub card_id: Option<i32>,
    pub current_health: Option<i32>,
}

rbatis::crud!(MatchCard {}, "match_cards");

///创建对战
pub async fn create_match(rb: &RBatis, player1_id: i32, player2_id: i32) -> Result<Match, rbatis::Error> {
    let new_match = Match {
        id: None,
        player1_id: Some(player1_id),
        player2_id: Some(player2_id),
        winner_id: None,
        status: Some("ongoing".to_string()),
        start_time: Some(rbatis::rbdc::datetime::DateTime::now()),
    };
    Match::insert(rb, &new_match).await?;
    Ok(new_match)
}