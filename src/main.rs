use anyhow::Result;
use reqwest::Client;
use sqlx::PgPool;

mod api;
mod leaderboard;
mod league;
mod model;
mod replay;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = PgPool::connect(&dotenvy::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    let client = Client::new();

    // loop {
    //     league::crawl(pool.clone(), client.clone()).await?;
    // }
    // leaderboard::update(pool.clone(), client.clone()).await?;

    replay::fetch(pool, client).await?;

    Ok(())
}
