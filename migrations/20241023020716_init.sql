create table league_leaderboard (
    user_id bytea primary key not null,

    games_played integer not null,
    games_won integer not null,

    rank smallint not null,
    best_rank smallint not null,

    tr real not null,
    glicko real not null,
    rd real not null,
    gxe real not null,
    decaying boolean not null,
    
    apm real not null,
    pps real not null,
    vs real not null
);

create table replay_raw (
    id bytea primary key not null,
    data bytea,
    indexed boolean not null default false
);

create index replay_to_fetch on replay_raw (id) where data is null;
create index replay_to_index on replay_raw (id) where not indexed;
