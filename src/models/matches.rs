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
pub async fn create_match(
    rb: &RBatis,
    player1_id: i32,
    player2_id: i32,
) -> Result<Match, rbatis::Error> {
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

/// 出牌
pub async fn play_card(
    rb: &RBatis,
    match_id: i32,
    player_id: i32,
    card_id: i32,
    initial_health: i32,
) -> Result<(), rbatis::Error> {
    let match_card = MatchCard {
        match_id: Some(match_id),
        player_id: Some(player_id),
        card_id: Some(card_id),
        current_health: Some(initial_health),
    };
    MatchCard::insert(rb, &match_card).await?;
    Ok(())
}

/// 攻击
pub async fn attack_card(
    rb: &RBatis,
    match_id: i32,
    attacker_card_id: i32,
    target_card_id: i32,
    damage: i32,
) -> Result<(), rbatis::Error> {
    rb.exec(
        "UPDATE match_cards SET current_health = current_health - $1 WHERE match_id = $2 AND card_id = $3",
        vec![damage.into(), match_id.into(), target_card_id.into()]
    ).await?;
    Ok(())
}

pub async fn get_match(rb: &RBatis, match_id: i32) -> Result<Match, rbatis::Error> {
    Match::select_by_column(rb, "id", match_id)
        .await
        .map(|mut v| v.pop().unwrap_or_default())
}

pub async fn get_match_cards(rb: &RBatis, match_id: i32) -> Result<Vec<MatchCard>, rbatis::Error> {
    MatchCard::select_by_column(rb, "match_id", match_id).await
}