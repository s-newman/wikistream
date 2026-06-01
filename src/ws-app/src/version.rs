const ENV_VERSION: Option<&'static str> = option_env!("WS_VERSION");
const FALLBACK_VERSION: &str = "Unknown-Version";

pub fn version() -> &'static str {
    // TODO: Use const unwrap_or when stable: https://github.com/rust-lang/rust/issues/143956
    ENV_VERSION.unwrap_or(FALLBACK_VERSION)
}
