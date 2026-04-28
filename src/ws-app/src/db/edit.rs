use crate::db::{DbError, Id};
use sqlx::PgExecutor;
use sqlx::types::Json;
use sqlx::types::chrono::NaiveDate;
use ws_models::Edit;

pub async fn create(conn: impl PgExecutor<'_>, event: Edit) -> Result<Id, DbError> {
    let (result,) = sqlx::query_as(
        r#"
        insert into edit_events
            (schema, namespace, title, title_url, comment, timestamp, username, bot, server_url, server_name, server_script_path, wiki, parsedcomment, meta_uri, meta_request_id, meta_id, meta_domain, meta_stream, meta_dt, meta_topic, meta_partition, meta_offset, id, notify_url, minor, length, revision)
        values
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)
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
        .bind(event.inner.id)
        .bind(event.inner.notify_url)
        .bind(event.inner.minor)
        .bind(Json(event.inner.length))
        .bind(Json(event.inner.revision))
        .fetch_one(conn)
        .await?;

    Ok(result)
}

pub async fn most_edited_on_date(
    conn: impl PgExecutor<'_>,
    date: NaiveDate,
) -> Result<Vec<(i64, String, String)>, DbError> {
    let result = sqlx::query_as(
        r#"
        select
            count(*) as total,
            title,
            title_url
        from edit_events
        where
            wiki = 'enwiki'
            and namespace in (0, 1)
            and meta_dt::timestamp::date = $1
        group by title, title_url
        order by total desc
        limit 10
        "#,
    )
    .bind(date)
    .fetch_all(conn)
    .await?;

    Ok(result)
}
