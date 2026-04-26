use crate::db::{DbError, Id};
use sqlx::PgExecutor;
use ws_models::Categorize;

pub async fn create(conn: impl PgExecutor<'_>, event: Categorize) -> Result<Id, DbError> {
    let (result,) = sqlx::query_as(
        r#"
        insert into categorize_events
            (schema, namespace, title, title_url, comment, timestamp, username, bot, server_url, server_name, server_script_path, wiki, parsedcomment, meta_uri, meta_request_id, meta_id, meta_domain, meta_stream, meta_dt, meta_topic, meta_partition, meta_offset)
        values
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
        returning event_id
        "#,
    )
        .bind(event.shared.schema)
        .bind(event.shared.namespace)
        .bind(event.shared.title)
        .bind(event.shared.title_url)
        .bind(event.shared.comment)
        .bind(event.shared.timestamp)
        .bind(event.shared.user)
        .bind(event.shared.bot)
        .bind(event.shared.server_url)
        .bind(event.shared.server_name)
        .bind(event.shared.server_script_path)
        .bind(event.shared.wiki)
        .bind(event.shared.parsedcomment)
        .bind(event.shared.meta.uri)
        .bind(event.shared.meta.request_id)
        .bind(event.shared.meta.id)
        .bind(event.shared.meta.domain)
        .bind(event.shared.meta.stream)
        .bind(event.shared.meta.dt)
        .bind(event.shared.meta.topic)
        .bind(event.shared.meta.partition)
        .bind(event.shared.meta.offset)
        .fetch_one(conn)
        .await?;

    Ok(result)
}
