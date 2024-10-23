use anyhow::Result;
use clap::{Parser, Subcommand};
use reqwest::Client;
use sqlx::PgPool;

mod api;
mod leaderboard;
mod league;
mod model;
mod replay;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    FetchReplays,
    FetchLeague,
    CrawlLeague,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let pool = PgPool::connect(&dotenvy::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    let client = Client::new();

    match cli.command {
        Command::CrawlLeague => league::crawl(pool, client).await?,
        Command::FetchLeague => leaderboard::update(pool, client).await?,
        Command::FetchReplays => replay::fetch(pool, client).await?,
    }

    Ok(())
}
