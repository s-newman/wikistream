pub mod abusefilter;
pub mod block;
pub mod communityrequests;
pub mod contentmodel;
pub mod delete;
pub mod gblblock;
pub mod gblrename;
pub mod gblrights;
pub mod globalauth;
pub mod growthexperiments;
pub mod import;
pub mod massmessage;
pub mod merge;
pub mod move_log;
pub mod newusers;
pub mod notifytranslators;
pub mod pagetranslation;
pub mod pagetriage_copyvio;
pub mod pagetriage_curation;
pub mod patrol;
pub mod protect;
pub mod renameuser;
pub mod review;
pub mod rights;
pub mod stable;
pub mod tag;
pub mod thanks;
pub mod translationreview;
pub mod upload;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Log {
    id: Option<u64>,
    log_id: u64,
    #[serde(flatten)]
    log_params: LogParams,
    log_action_comment: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_type")]
#[serde(rename_all = "lowercase")]
pub enum LogParams {
    Delete(delete::Params),
    Upload(upload::Params),
    AbuseFilter(abusefilter::Params),
    Block(block::Params),
    Patrol(patrol::Params),
    Move(move_log::Params),
    NewUsers(newusers::Params),
    Review(review::Params),
    Thanks(thanks::Params),
    GrowthExperiments(growthexperiments::Params),
    #[serde(rename = "pagetriage-curation")]
    PageTriageCuration(pagetriage_curation::Params),
    RenameUser(renameuser::Params),
    Protect(protect::Params),
    GlobalAuth(globalauth::Params),
    Import(import::Params),
    Rights(rights::Params),
    PageTranslation(pagetranslation::Params),
    TranslationReview(translationreview::Params),
    GblBlock(gblblock::Params),
    Stable(stable::Params),
    #[serde(rename = "pagetriage-copyvio")]
    PageTriageCopyVio(pagetriage_copyvio::Params),
    ContentModel(contentmodel::Params),
    Tag(tag::Params),
    GblRights(gblrights::Params),
    CommunityRequests(communityrequests::Params),
    NotifyTranslators(notifytranslators::Params),
}

// ===== Shared Types =====

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum MultiType {
    String(String),
    Number(u32),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Metadata {
    List(Vec<String>),
    Expiry { expiry: String },
}
