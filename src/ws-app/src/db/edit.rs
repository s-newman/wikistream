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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::str::FromStr;

    use super::*;
    use sqlx::PgPool;
    use tokio::fs::File;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use ws_models::{Event, FullEvent};

    async fn load_sample_edits() -> Vec<Edit> {
        let testdata_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            // Get to workspace root
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            // Then tests/data
            .join("tests")
            .join("data");
        let mut lines = BufReader::new(
            File::open(testdata_dir.join("edit_events_10.jsonl"))
                .await
                .unwrap(),
        )
        .lines();

        let mut result = Vec::new();

        while let Some(line) = lines.next_line().await.unwrap() {
            let event = Event::from_str(&line).unwrap();
            let Event::Event(FullEvent::Edit(edit)) = event else {
                panic!("edit_events_10.json contained non-edit events");
            };
            result.push(edit);
        }

        result
    }

    async fn count_edits(db: &PgPool) -> i64 {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            select
                count(*) as total
            from edit_events
            "#,
        )
        .fetch_one(db)
        .await
        .unwrap();

        count
    }

    #[sqlx::test]
    async fn test_create(db: PgPool) {
        let edits = load_sample_edits().await;

        let id = create(&db, edits[0].clone()).await;

        assert!(id.is_ok());
    }

    #[sqlx::test]
    async fn test_bulk_create(db: PgPool) {
        let edits = load_sample_edits().await;

        let ids = bulk_create(&db, edits).await;
        assert!(ids.is_ok());

        let count = count_edits(&db).await;
        assert_eq!(count, 10);
    }

    /// Bulk creating events should silently skip creating any events that
    /// already exist in the database and not throw an error.
    #[sqlx::test]
    async fn test_bulk_create_conflicts(db: PgPool) {
        let edits = load_sample_edits().await;
        bulk_create(&db, edits.clone()).await.unwrap();
        let first_count = count_edits(&db).await;

        let result = bulk_create(&db, edits).await;
        assert!(result.is_ok());

        let second_count = count_edits(&db).await;
        assert_eq!(first_count, second_count);
    }

    #[sqlx::test(fixtures("edit_events"))]
    async fn test_most_edited_on_date(db: PgPool) {
        let date = NaiveDate::from_ymd_opt(2026, 5, 28).unwrap();

        let top_25 = most_edited_on_date(&db, &date).await.unwrap();

        assert_eq!(top_25[0].title, "Test Page");
    }

    /// Checking the most edited pages on a date that has no edits in the
    /// database should return an empty list.
    #[sqlx::test(fixtures("edit_events"))]
    async fn test_most_edited_on_date_empty(db: PgPool) {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let top_25 = most_edited_on_date(&db, &date).await.unwrap();

        assert!(top_25.is_empty());
    }
}
