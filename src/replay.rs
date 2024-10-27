use std::time::Duration;

use anyhow::{bail, Result};
use reqwest::{Client, StatusCode};
use sqlx::{query, query_scalar, PgPool};
use tokio::time::sleep;
use zstd::encode_all;

pub async fn fetch(pool: PgPool, client: Client) -> Result<()> {
    loop {
        let mut tx = pool.begin().await?;
        let id = match query_scalar!(
            "select id from replay_queue order by priority asc nulls last for update skip locked"
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            Some(x) => x,
            None => break,
        };

        let id_hex = hex::encode(&id);
        let url = format!("https://inoue.szy.lol/api/replay/{id_hex}");
        let response = client.get(&url).send().await?;

        let status = response.status();
        println!("{url} {status}");
        if status == StatusCode::OK {
            let data = response.bytes().await?;
            let data = encode_all(&*data, 19)?;
            query!(
                "insert into replay_raw (id, data) values ($1, $2)",
                id,
                data
            )
            .execute(&mut *tx)
            .await?;
        } else if status == StatusCode::TOO_MANY_REQUESTS {
            continue;
        } else if status != StatusCode::NOT_FOUND {
            bail!("unexpected status code: {status}")
        }
        query!("delete from replay_queue where id = $1", id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        sleep(Duration::from_secs(10)).await;
    }

    Ok(())
}
