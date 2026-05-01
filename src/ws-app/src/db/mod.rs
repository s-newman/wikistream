use anyhow::Context;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::time::Duration;
use uuid::Uuid;

pub mod edit;

pub type Pool = sqlx::PgPool;

pub type Id = Uuid;

/// Database connection settings.
pub struct DbConfig {
    connect_opts: PgConnectOptions,
    max_connections: u32,
    migrate: bool,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            connect_opts: PgConnectOptions::default(),
            max_connections: 50,
            migrate: true,
        }
    }
}

impl DbConfig {
    /// Create a new configuration. This is equivalent to calling [DbConfig::default()].
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether to apply database migrations on startup.
    ///
    /// **Default**: `true`
    pub fn migrate(self, migrate: bool) -> Self {
        Self { migrate, ..self }
    }
}

/// Open a database connection pool with the given connection options.
pub async fn connect(config: &DbConfig) -> anyhow::Result<Pool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        // Default timeout is 30 seconds, which is frustrating if the database isn't running --
        // dropping the connect timeout gets us that feedback much faster.
        //
        // May consider increasing this timeout after the initial connection is established if it
        // seems necessary after stress testing.
        .acquire_timeout(Duration::from_secs(3))
        .connect_with(config.connect_opts.clone())
        .await
        .context("failed to connect to database")?;

    if config.migrate {
        sqlx::migrate!()
            .run(&pool)
            .await
            .context("failed to run database migrations")?;
    }

    Ok(pool)
}

/// Run a basic SQL query to confirm the database connection.
pub async fn ping(db_pool: &Pool) -> Result<(), sqlx::Error> {
    sqlx::query("select 'pong' as ping")
        .execute(db_pool)
        .await?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("requested operation would violate uniqueness constraint")]
    Conflict,

    #[error("database row not found")]
    NotFound,

    #[error("unknown error")]
    Unknown(sqlx::Error),
}

impl From<sqlx::Error> for DbError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::Database(e) if e.is_unique_violation() => Self::Conflict,
            sqlx::Error::RowNotFound => Self::NotFound,
            e => {
                tracing::error!(error=%e, "unexpected sqlx error");
                Self::Unknown(e)
            }
        }
    }
}
