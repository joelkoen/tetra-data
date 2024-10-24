create table league_match (
    replay_id bytea primary key not null,
    timestamp timestamp with time zone not null,
    results jsonb not null
);

update league set games_to_crawl = games_played, last_crawled = null;
