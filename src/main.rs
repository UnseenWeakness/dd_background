mod config;

use axum::routing::get;
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

// 玩家数据结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Players {
    pub id: Option<i32>,
    pub nickname: Option<String>,
    pub wins: Option<i32>,
}

rbatis::crud!(Players {});

// 获取所有玩家
async fn get_players(axum::extract::State(rb): axum::extract::State<RBatis>) -> Json<Vec<Players>> {
    let players = Players::select_all(&rb).await.unwrap_or_default();
    Json(players)
}
