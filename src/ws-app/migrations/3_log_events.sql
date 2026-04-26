create table log_events
(
    event_id           uuid primary key default gen_random_uuid(),

    -- shared fields
    schema             text        not null,
    namespace          int         not null,
    title              text        not null,
    title_url          text        not null,
    comment            text        not null,
    timestamp          bigint      not null,
    username           text        not null,
    bot                boolean     not null,
    server_url         text        not null,
    server_name        text        not null,
    server_script_path text        not null,
    wiki               text        not null,
    parsedcomment      text        not null,

    -- metadata fields from $.meta
    meta_uri           text        not null,
    meta_request_id    text        not null,
    meta_id            text        not null,
    meta_domain        text        not null,
    meta_stream        text        not null,
    meta_dt            timestamptz not null,
    meta_topic         text        not null,
    meta_partition     integer     not null,
    meta_offset        bigint      not null,

    -- fields specific to log events
    id                 bigint,
    log_id             bigint      not null,
    log_params         json        not null,
    log_action_comment text        not null
);