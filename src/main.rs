mod config;
mod models;

use crate::models::cards::Card;
use crate::models::players::Players;
use axum::extract::Path;
use axum::routing::{get, post};
use axum::{Json, Router};
use log::LevelFilter;
use rbatis::dark_std::defer;
use rbatis::RBatis;
use serde::{Deserialize, Serialize};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    _ = fast_log::init(fast_log::Config::new().console().level(LevelFilter::Debug));
    defer!(|| {
        log::logger().flush();
    });
    // 加载配置文件
    let config = config::load_config("config.yaml")?;
    let connect = format!(
        "postgresql://{}:{}@{}:{}/{}",
        config.database.username,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.dbname
    );
    let rb = RBatis::new();
    rb.init(rbdc_pg::driver::PgDriver {}, &connect).unwrap();

    // 定义路由
    let app = Router::new()
        .route("/", get(root))
        .route("/players", get(get_players))
        .route("/cards", get(get_cards))
        .route("/players/{id}/cards", get(get_player_cards))
        .route("/players/{id}/cards", post(add_card_to_player))
        .route("/matches", post(create_match))
        .with_state(rb); // 传递 rbatis 实例

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:46301").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// 根路由
async fn root() -> &'static str {
    "欢迎来到卡牌对战游戏！"
}

// 获取所有玩家
async fn get_players(axum::extract::State(rb): axum::extract::State<RBatis>) -> Json<Vec<Players>> {
    let players = models::players::get_all(&rb).await.unwrap_or_default();
    Json(players)
}

// 获取所有卡牌模板
async fn get_cards(axum::extract::State(rb): axum::extract::State<RBatis>) -> Json<Vec<Card>> {
    let cards = models::cards::get_all(&rb).await.unwrap_or_default();
    Json(cards)
}

// 获取玩家卡组
async fn get_player_cards(
    axum::extract::State(rb): axum::extract::State<RBatis>,
    Path(player_id): Path<i32>,
) -> Json<Vec<Card>> {
    let cards = models::player_cards::get_by_player(&rb, player_id)
        .await
        .unwrap_or_default();
    Json(cards)
}

#[derive(Deserialize)]
struct AddCardRequest {
    card_id: i32,
}

async fn add_card_to_player(
    axum::extract::State(rb): axum::extract::State<RBatis>,
    Path(player_id): Path<i32>,
    Json(request): Json<AddCardRequest>,
) -> String {
    match models::player_cards::add_card(&rb, player_id, request.card_id).await {
        Ok(_) => "卡牌添加成功".to_string(),
        Err(e) => format!("添加失败: {}", e),
    }
}

#[derive(Deserialize)]
struct CreateMatchRequest {
    player1_id: i32,
    player2_id: i32,
}

async fn create_match(
    axum::extract::State(rb): axum::extract::State<RBatis>,
    Json(request): Json<CreateMatchRequest>,
) -> Json<models::matches::Match> {
    let new_match = models::matches::create_match(&rb, request.player1_id, request.player2_id)
        .await
        .unwrap_or_else(|e| models::matches::Match {
            status: Some(format!("创建失败: {}", e)),
            ..Default::default()
        });

    Json(new_match)
}
