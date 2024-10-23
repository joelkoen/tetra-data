create table league (
    user_id bytea primary key not null,

    games_played integer not null,
    games_won integer not null,

    placement integer,
    rank smallint not null,
    best_rank smallint not null,

    tr real not null,
    glicko real not null,
    rd real not null,
    gxe real not null,
    decaying boolean not null,
    
    apm real not null,
    pps real not null,
    vs real not null,

    games_to_crawl integer not null default 0,
    last_crawled timestamp with time zone
);

create index league_placement on league (placement) where placement is not null;

create table replay_queue (
    id bytea primary key not null,
    priority integer
);

create index replay_queue_priority on replay_queue (priority asc nulls last, id);

create table replay_raw (
    id bytea primary key not null,
    data bytea not null
);
