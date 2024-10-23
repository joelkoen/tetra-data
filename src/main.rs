use anyhow::{bail, Result};
use reqwest::{Client, StatusCode};
use sqlx::{query, query_scalar, PgPool};

#[tokio::main]
async fn main() -> Result<()> {
    let pool = PgPool::connect(&dotenvy::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    let client = Client::new();
    loop {
        let mut tx = pool.begin().await?;
        let id = match query_scalar!("select id from replay_raw where data is null")
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
            query!(
                "update replay_raw set data = $1, indexed = false where id = $2",
                &*data,
                id
            )
            .execute(&mut *tx)
            .await?;
        } else if status == StatusCode::NOT_FOUND {
            query!("delete from replay_raw where id = $1", id)
                .execute(&mut *tx)
                .await?;
        } else {
            bail!("unexpected status code: {status}")
        }
        tx.commit().await?;
    }

    Ok(())
}
