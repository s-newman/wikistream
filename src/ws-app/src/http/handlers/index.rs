use crate::http::responses::HttpError;
use crate::http::server::AppState;
use crate::views::{DailyArgs, Page};
use crate::{db, views};
use anyhow::Context;
use axum::extract::State;
use axum::response::Html;
use chrono::{Days, Utc};

pub(super) async fn handler(State(app_state): State<AppState>) -> Result<Html<String>, HttpError> {
    let date = Utc::now().date_naive();

    let pages: Vec<Page> = db::edit::most_edited_on_date(&app_state.db_pool, &date)
        .await
        .context("failed to fetch from database")?
        .iter()
        .map(|x| x.into())
        .collect();

    let tmpl_args = DailyArgs {
        date,
        pages,
        previous_day: date.checked_sub_days(Days::new(1)),
        next_day: date.checked_add_days(Days::new(1)),
    };
    let out = views::daily(&app_state.env, tmpl_args).context("failed to render template")?;

    Ok(Html(out))
}
