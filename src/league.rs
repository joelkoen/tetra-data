use std::collections::BTreeSet;

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use sqlx::{query, PgPool};

use crate::api::{ApiEntriesOf, ApiResponse};

pub async fn crawl(pool: PgPool, client: Client) -> Result<()> {
    let mut tx = pool.begin().await?;
    let user =
        query!("select user_id, placement, last_crawled from league where games_to_crawl > 10 order by placement for update")
            .fetch_one(&pool)
            .await?;
    let user_id = hex::encode(&user.user_id);

    let mut replays = BTreeSet::new();
    let mut after = user.last_crawled.unwrap_or_default();
    loop {
        let response: ApiEntriesOf<Record> =
            client.get(format!("https://ch.tetr.io/api/users/{user_id}/records/league/recent?limit=100&before={}:0:0", after.timestamp_millis())).header("x-session-id","meow-2" ).send().await?.json().await?;

        let mut entries = match response {
            ApiResponse::Error { error } => bail!(error.msg),
            ApiResponse::Success { data, .. } => data.entries,
        };
        entries.sort_by_key(|x| x.timestamp);

        if let Some(last) = entries.last() {
            after = last.timestamp;
        }

        let len = entries.len();
        for record in entries {
            if !record.stub {
                replays.insert(record.replay_id);
            }
        }
        dbg!(replays.len());

        if len < 100 {
            break;
        }
    }

    query!(
        "update league set games_to_crawl = 0, last_crawled = $1 where user_id = $2",
        after,
        user.user_id
    )
    .execute(&mut *tx)
    .await?;
    for replay in replays {
        let id = hex::decode(replay)?;
        query!(
            "insert into replay_queue (id, priority) values ($1, $2) on conflict do nothing",
            id,
            user.placement
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct Record {
    stub: bool,
    #[serde(rename = "replayid")]
    replay_id: String,
    #[serde(rename = "ts")]
    timestamp: DateTime<Utc>,
}
