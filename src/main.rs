use std::{
    io::{stdout, Write},
    time::Duration,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::Env;
use reqwest::Client;
use sqlx::{query_scalar, PgPool};

mod api;
mod crawl_league;
mod fetch_league;
mod fetch_replays;
mod model;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    FetchLeague,
    CrawlLeague,
    FetchReplays,
    DumpReplays,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::init_from_env(Env::new().default_filter_or("info"));

    let pool = PgPool::connect(&dotenvy::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    let client = Client::builder()
        .user_agent("discord @joelllllllllllllllllllllllllllll")
        .timeout(Duration::from_secs(15))
        .build()?;

    match cli.command {
        Command::FetchLeague => fetch_league::run(pool, client).await?,
        Command::CrawlLeague => crawl_league::run(pool, client).await?,
        Command::FetchReplays => fetch_replays::run(pool, client).await?,
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
