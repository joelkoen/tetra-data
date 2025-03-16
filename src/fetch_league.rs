use std::{collections::BTreeSet, fmt};

use anyhow::{bail, Result};
use reqwest::Client;
use serde::Deserialize;
use sqlx::{query, PgPool};

use crate::{
    api::EntriesOf,
    model::{ObjectId, Rank},
};

pub async fn run(pool: PgPool, client: Client) -> Result<()> {
    let mut tx = pool.begin().await?;

    let mut data = Vec::new();
    let mut seen = BTreeSet::new();
    let mut after: Option<Prisecter> = None;
    loop {
        let request = client.get("https://ch.tetr.io/api/users/by/league?limit=100");
        let request = match &after {
            Some(x) => request.query(&[("after", &*x.to_string())]),
            None => request,
        };

        let response: EntriesOf<LeagueRecord> = request
            .header("x-session-id", "meow")
            .send()
            .await?
            .json()
            .await?;
        let entries = response.data.entries;

        if let Some(last) = entries.last() {
            let mut p = last.prisecter.clone();
            p.pri += 1.0; // float formatting means this is unreliable
            after = Some(p);
        }

        let len = entries.len();
        for entry in entries {
            if seen.insert(entry.user_id) {
                data.push((entry.user_id, entry.league));
            }
        }
        dbg!(data.len());

        if len < 10 {
            break;
        }
    }

    query!("update league set placement = null")
        .execute(&mut *tx)
        .await?;
    for (placement, (user_id, data)) in data.into_iter().enumerate() {
        let placement = placement + 1;
        let LeagueData {
            games_played,
            games_won,
            rank,
            best_rank,
            tr,
            glicko,
            rd,
            gxe,
            decaying,
            apm,
            pps,
            vs,
        } = data;
        query!(
            "
                insert into league
                    (user_id, games_played, games_won, placement, rank, best_rank, tr, glicko, rd, gxe, decaying, apm, pps, vs, games_to_crawl)
                values
                    ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $2)
                on conflict (user_id) do update set
                    games_played = excluded.games_played,
                    games_won = excluded.games_won,
                    placement = excluded.placement,
                    rank = excluded.rank,
                    best_rank = excluded.best_rank,
                    tr = excluded.tr,
                    glicko = excluded.glicko,
                    rd = excluded.rd,
                    gxe = excluded.gxe,
                    decaying = excluded.decaying,
                    apm = excluded.apm,
                    pps = excluded.pps,
                    vs = excluded.vs,
                    games_to_crawl = league.games_to_crawl + (excluded.games_played - league.games_played)
            ",
            &user_id.0,
            games_played as i32,
            games_won as i32,
            placement as i32,
            rank as i16,
            best_rank as i16,
            tr,
            glicko,
            rd,
            gxe,
            decaying,
            apm.unwrap_or_default(),
            pps.unwrap_or_default(),
            vs.unwrap_or_default()
        ).execute(&mut *tx).await?;
    }
    tx.commit().await?;

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct LeagueRecord {
    #[serde(rename = "_id")]
    user_id: ObjectId,
    league: LeagueData,
    #[serde(rename = "p")]
    prisecter: Prisecter,
}

#[derive(Debug, Clone, Deserialize)]
struct LeagueData {
    #[serde(rename = "gamesplayed")]
    games_played: u32,
    #[serde(rename = "gameswon")]
    games_won: u32,
    rank: Rank,
    #[serde(rename = "bestrank")]
    best_rank: Rank,
    tr: f32,
    glicko: f32,
    rd: f32,
    gxe: f32,
    decaying: bool,
    apm: Option<f32>,
    pps: Option<f32>,
    vs: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct Prisecter {
    pri: f64,
    sec: f64,
    ter: f64,
}

impl fmt::Display for Prisecter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.pri, self.sec, self.ter)
    }
}
