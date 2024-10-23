create table replay_raw (
    id bytea primary key not null,
    data bytea,
    indexed boolean not null default false
);

create index replay_to_fetch on replay_raw (id) where data is null;
create index replay_to_index on replay_raw (id) where not indexed;
