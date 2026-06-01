use crate::db::{DbError, Id};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgExecutor, Postgres, QueryBuilder};
use ws_models::Edit;

pub async fn create(conn: impl PgExecutor<'_>, event: Edit) -> Result<Id, DbError> {
    let (result,) = sqlx::query_as(
        r#"
        insert into edit_events (
            namespace,
            title,
            title_url,
            timestamp,
            username,
            wiki,
            meta_request_id,
            meta_id,
            meta_dt,
            meta_dt_date
        )
        values
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        returning event_id
        "#,
    )
    .bind(event.shared.namespace)
    .bind(event.shared.title)
    .bind(event.shared.title_url)
    .bind(event.shared.timestamp)
    .bind(event.shared.user)
    .bind(event.shared.wiki)
    .bind(event.shared.meta.request_id)
    .bind(event.shared.meta.id)
    .bind(event.shared.meta.dt)
    .bind(event.shared.meta.dt.date_naive())
    .fetch_one(conn)
    .await?;

    Ok(result)
}

pub async fn bulk_create(conn: impl PgExecutor<'_>, events: Vec<Edit>) -> Result<(), DbError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        insert into edit_events (
            namespace,
            title,
            title_url,
            timestamp,
            username,
            wiki,
            meta_request_id,
            meta_id,
            meta_dt,
            meta_dt_date
        )
        "#,
    );
    qb.push_values(events, |mut b, event| {
        b.push_bind(event.shared.namespace)
            .push_bind(event.shared.title)
            .push_bind(event.shared.title_url)
            .push_bind(event.shared.timestamp)
            .push_bind(event.shared.user)
            .push_bind(event.shared.wiki)
            .push_bind(event.shared.meta.request_id)
            .push_bind(event.shared.meta.id)
            .push_bind(event.shared.meta.dt)
            .push_bind(event.shared.meta.dt.date_naive());
    });
    qb.push(
        r#"
        on conflict do nothing
        "#,
    );
    qb.build().fetch_all(conn).await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct EditPageView {
    pub total: i64,
    pub editors: i64,
    pub title: String,
    pub title_url: String,
}

pub async fn most_edited_on_date(
    conn: impl PgExecutor<'_>,
    date: &NaiveDate,
) -> Result<Vec<EditPageView>, DbError> {
    let result = sqlx::query_as(
        r#"
        select
            count(*) as total,
            count(distinct username) as editors,
            title,
            title_url
        from edit_events
        where
            namespace in (0, 1)
            and meta_dt_date = $1
        group by title, title_url
        order by total desc
        limit 25
        "#,
    )
    .bind(date)
    .fetch_all(conn)
    .await?;

    Ok(result)
}

pub async fn most_edited_between(
    conn: impl PgExecutor<'_>,
    start: &DateTime<Utc>,
    end: &DateTime<Utc>,
) -> Result<Vec<EditPageView>, DbError> {
    let result = sqlx::query_as(
        r#"
        select
            count(*) as total,
            count(distinct username) as editors,
            title,
            title_url
        from edit_events
        where
            namespace in (0, 1)
            and meta_dt > $1
            and meta_dt < $2
        group by title, title_url
        order by total desc
        limit 25
        "#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(conn)
    .await?;

    Ok(result)
}
