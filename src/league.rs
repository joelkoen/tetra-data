use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use sqlx::{query, query_scalar, PgPool};
use tokio::time::sleep;

use crate::api::EntriesOf;

pub async fn crawl(pool: PgPool, client: Client) -> Result<()> {
    loop {
        let mut tx = pool.begin().await?;
        let user =
        if let Some(user) = 
            // uncrawled users
            query_scalar!("select user_id from league where placement is not null and last_crawled is null order by placement for update skip locked")
                .fetch_optional(&mut *tx)
                .await? {
            user
        } else if let Some(user) =  
            // recrawling new games
            query_scalar!("select user_id from league where placement is not null and games_to_crawl > 0 order by placement for update skip locked")
                .fetch_optional(&mut *tx)
                .await? {
            user
        } else {
            break;
        };

        let record = query!("select placement, games_to_crawl, last_crawled from league where user_id = $1", user).fetch_one(&mut *tx).await?;
        let user_id = hex::encode(&user);
        let placement = record.placement.unwrap();
        println!(
            "crawling {} from {user_id} (#{placement})",
            record.games_to_crawl
        );

        let mut replays = BTreeSet::new();
        let mut matches = BTreeMap::new();
        let mut after = record.last_crawled.unwrap_or_default();
        loop {
            let response: EntriesOf<Record> =
            client.get(format!("https://ch.tetr.io/api/users/{user_id}/records/league/recent?limit=100&before={}:0:0", after.timestamp_millis())).header("x-session-id","meow-2" ).send().await?.json().await?;

            let mut entries = response.data.entries;
            entries.sort_by_key(|x| x.timestamp);

            if let Some(last) = entries.last() {
                after = last.timestamp;
            }

            let len = entries.len();
            for record in entries {
                let id = hex::decode(record.replay_id)?;
                if !record.stub && placement <= 1000 {
                    replays.insert(id.clone());
                }

                let results = record.results;
                matches.insert(
                    id,
                    (
                        record.timestamp,
                        json!({
                            "leaderboard": results.leaderboard,
                            "rounds": results.rounds,
                            "league": record.extras.league
                        }),
                    ),
                );
            }

            if len < 100 {
                break;
            }
        }

        query!(
            "update league set games_to_crawl = 0, last_crawled = $1 where user_id = $2",
            after,
            user
        )
        .execute(&mut *tx)
        .await?;
        dbg!(replays.len(), matches.len());
        for id in replays {
            let exists = query!("select 1 as x from replay_raw where id = $1", id)
                .fetch_optional(&mut *tx)
                .await?
                .is_some();
            if !exists {
                query!(
                "insert into replay_queue (id, priority) values ($1, $2) on conflict do nothing",
                id,
                placement
            )
                .execute(&mut *tx)
                .await?;
            }
        }
        for (id, (timestamp, results)) in matches {
            query!("insert into league_match (replay_id, timestamp, results) values ($1, $2, $3) on conflict do nothing",id, timestamp, results ).execute(&mut *tx).await?;
        }
        tx.commit().await?;

        if placement > 10000 {
            sleep(Duration::from_secs(2)).await;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct Record {
    stub: bool,
    #[serde(rename = "replayid")]
    replay_id: String,
    #[serde(rename = "ts")]
    timestamp: DateTime<Utc>,
    results: RecordResults,
    extras: RecordExtra,
}

#[derive(Debug, Clone, Deserialize)]
struct RecordResults {
    leaderboard: serde_json::Value,
    rounds: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct RecordExtra {
    league: BTreeMap<String, serde_json::Value>,
}
