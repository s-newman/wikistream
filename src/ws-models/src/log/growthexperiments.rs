use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    #[serde(rename_all = "kebab-case")]
    SetMentor {
        previous_mentor: String,
        new_mentor: String,
    },
    AddLink {
        count: u8,
    },
    AddImage {
        accepted: bool,
    },
    AddSectionImage {
        section: String,
        accepted: bool,
    },
    #[serde(rename_all = "kebab-case")]
    ClaimMentee {
        previous_mentor: String,
    },
    #[serde(rename = "claimmentee-no-previous-mentor")]
    ClaimMenteeNoPreviousMentor(Vec<String>),
}
