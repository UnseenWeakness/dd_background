use serde::{Deserialize, Serialize};
use rbatis::RBatis;
use crate::models::cards::Card;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerCards {
    pub player_id: Option<i32>,
    pub card_id: Option<i32>,
}

rbatis::crud!(PlayerCards {});

pub async fn get_by_player(rb: &RBatis, player_id: i32) -> Result<Vec<Card>, rbatis::Error> {
    rb.query_decode(
        "SELECT c.* FROM cards c JOIN player_cards pc ON c.id = pc.card_id WHERE pc.player_id = $1",
        vec![player_id.into()]
    ).await
}

pub async fn add_card(rb: &RBatis, player_id: i32, card_id: i32) -> Result<(), rbatis::Error> {
    rb.exec(
        "INSERT INTO player_cards (player_id, card_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        vec![player_id.into(), card_id.into()]
    ).await?;
    Ok(())
}