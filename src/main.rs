use std::{
    io::{stdout, Write},
    time::Duration,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use reqwest::Client;
use sqlx::{query_scalar, PgPool};

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
    DumpReplays,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let pool = PgPool::connect(&dotenvy::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    let client = Client::builder()
        .user_agent("discord @joelllllllllllllllllllllllllllll")
        .timeout(Duration::from_secs(15))
        .build()?;

    match cli.command {
        Command::CrawlLeague => league::crawl(pool, client).await?,
        Command::FetchLeague => leaderboard::update(pool, client).await?,
        Command::FetchReplays => replay::fetch(pool, client).await?,
        Command::DumpReplays => {
            let replays = query_scalar!("select data from replay_raw")
                .fetch_all(&pool)
                .await?;
            let mut out = stdout();
            for replay in replays {
                out.write_all(&zstd::decode_all(&*replay)?)?;
                out.write_all(&[b'\n'])?;
            }
        }
    }

    Ok(())
}
