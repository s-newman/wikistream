use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const STATE_FILE: &str = ".ws-ingest-state.json";
const TEMP_STATE_FILE: &str = ".tmp.ws-ingest-state.json";

#[derive(Serialize, Deserialize)]
pub struct IngestState {
    pub last_ingested_ms: i64,
}

impl IngestState {
    pub fn load<P: AsRef<Path>>(data_dir: P) -> anyhow::Result<Option<Self>> {
        let state_file = data_dir.as_ref().join(STATE_FILE);
        if !state_file.exists() {
            return Ok(None);
        }

        let state_file_content =
            fs::read_to_string(state_file).context("failed to read ingest state file")?;
        Ok(Some(
            serde_json::from_str::<Self>(&state_file_content)
                .context("failed to deserialize state file")?,
        ))
    }

    pub fn save<P: AsRef<Path>>(&self, data_dir: P) -> anyhow::Result<()> {
        let content = serde_json::to_vec(self).context("failed to serialize ingest state")?;
        let tmp_state_file = data_dir.as_ref().join(TEMP_STATE_FILE);
        fs::write(&tmp_state_file, content).context("failed to write ingest state to temp file")?;
        let state_file = data_dir.as_ref().join(STATE_FILE);
        fs::rename(tmp_state_file, state_file).context("failed to rename temp state file")
    }
}

impl From<DateTime<Utc>> for IngestState {
    fn from(value: DateTime<Utc>) -> Self {
        Self {
            last_ingested_ms: value.timestamp_millis(),
        }
    }
}
