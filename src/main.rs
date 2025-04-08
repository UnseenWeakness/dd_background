mod config;
mod models;

use std::sync::Arc;
use crate::models::cards::Card;
use crate::models::players::Players;
use axum::extract::{Path, WebSocketUpgrade};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use fast_log::TimeType::Utc;
use log::LevelFilter;
use moka::future::Cache;
use rbatis::dark_std::defer;
use rbatis::RBatis;
use rbatis::rbdc::DateTime;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

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

    // 创建广播通道，用于同步对战状态
    let (tx, _rx) = broadcast::channel(100);

    let card_cache = Arc::new(Cache::new(10)); // 缓存最多 10 个条目

    // 定义路由
    let app = Router::new()
        .route("/", get(root))
        .route("/info", get(info))
        .route("/players", get(get_players))
        .route("/cards", get(get_cards))
        .route("/players/{id}/cards", get(get_player_cards))
        .route("/players/{id}/cards", post(add_card_to_player))
        .route("/matches", post(create_match))
        .route("/matches/{id}/play", post(play_card))
        .route("/matches/{id}/attack", post(attack_card))
        .route("/matches/{id}/ws", get(ws_handler))
        .route("/matches/{id}/test", post(test_websocket)) // 新增测试接口
        .with_state(AppState { rb, tx, card_cache });

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:46301").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// 应用状态，包含 RBatis 和广播通道
#[derive(Clone)]
struct AppState {
    rb: RBatis,
    tx: broadcast::Sender<String>,
    card_cache: Arc<Cache<String, Vec<Card>>>,
}

// 根路由
async fn root() -> &'static str {
    "欢迎来到卡牌对战游戏！"
}

async fn info() -> &'static str {
    "这是个刺激的对战游戏！"
}

// 获取所有玩家
async fn get_players(axum::extract::State(state): axum::extract::State<AppState>) -> Json<Vec<Players>> {
    let players = models::players::get_all(&state.rb).await.unwrap_or_default();
    Json(players)
}

// 获取所有卡牌模板
async fn get_cards(axum::extract::State(state): axum::extract::State<AppState>) -> Json<Vec<Card>> {
    let cards = models::cards::get_all(&state.rb, state.card_cache.clone()).await.unwrap_or_default();
    Json(cards)
}

// 获取玩家卡组
async fn get_player_cards(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(player_id): Path<i32>,
) -> Json<Vec<Card>> {
    let cards = models::player_cards::get_by_player(&state.rb, player_id)
        .await
        .unwrap_or_default();
    Json(cards)
}

#[derive(Deserialize)]
struct AddCardRequest {
    card_id: i32,
}

async fn add_card_to_player(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(player_id): Path<i32>,
    Json(request): Json<AddCardRequest>,
) -> String {
    match models::player_cards::add_card(&&state.rb, player_id, request.card_id).await {
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
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(request): Json<CreateMatchRequest>,
) -> Json<models::matches::Match> {
    let new_match = models::matches::create_match(&state.rb, request.player1_id, request.player2_id)
        .await
        .unwrap_or_else(|e| models::matches::Match {
            status: Some(format!("创建失败: {}", e)),
            ..Default::default()
        });

    Json(new_match)
}

// 新增处理器
#[derive(Deserialize)]
struct PlayCardRequest {
    player_id: i32,
    card_id: i32,
    initial_health: i32,
}

async fn play_card(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(match_id): Path<i32>,
    Json(request): Json<PlayCardRequest>,
) -> String {
    match models::matches::play_card(
        &state.rb,
        match_id,
        request.player_id,
        request.card_id,
        request.initial_health,
    )
    .await
    {
        Ok(_) => "出牌成功".to_string(),
        Err(e) => format!("出牌失败: {}", e),
    }
}

#[derive(Deserialize)]
struct AttackCardRequest {
    attacker_card_id: i32,
    target_card_id: i32,
    damage: i32,
}

async fn attack_card(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(match_id): Path<i32>,
    Json(request): Json<AttackCardRequest>,
) -> String {
    match models::matches::attack_card(
        &state.rb,
        match_id,
        request.attacker_card_id,
        request.target_card_id,
        request.damage,
    )
    .await
    {
        Ok(_) => "攻击成功".to_string(),
        Err(e) => format!("攻击失败: {}", e),
    }
}

// WebSocket 处理器
async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(match_id): Path<i32>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, match_id))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, match_id: i32) {
    let mut rx = state.tx.subscribe();
    // 发送初始对战状态
    let match_data = models::matches::get_match(&state.rb, match_id).await.unwrap_or_default();
    let match_cards = models::matches::get_match_cards(&state.rb, match_id).await.unwrap_or_default();
    let initial_state = serde_json::to_string(&MatchState { match_data, match_cards }).unwrap();
    // 将 String 转换为 Utf8Bytes
    let initial_state_bytes = Utf8Bytes::from(initial_state);
    let _ = socket.send(Message::Text(initial_state_bytes)).await;

    // 监听广播消息
    while let Ok(msg) = rx.recv().await {
        // 将 String 转换为 Utf8Bytes
        let msg_bytes = Utf8Bytes::from(msg);
        let _ = socket.send(Message::Text(msg_bytes)).await;
    }
}

#[derive(Serialize)]
struct MatchState {
    match_data: models::matches::Match,
    match_cards: Vec<models::matches::MatchCard>,
}

// 新增测试接口
async fn test_websocket(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(match_id): Path<i32>,
) -> String {
    let test_msg = format!("Test message for match {} at {}", match_id, DateTime::now());
    match state.tx.send(test_msg.clone()) {
        Ok(_) => {
            log::info!("Test message sent: {}", test_msg);
            "测试消息已发送".to_string()
        }
        Err(e) => {
            log::error!("Failed to send test message: {}", e);
            format!("测试消息发送失败: {}", e)
        }
    }
}