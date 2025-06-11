use once_cell::sync::OnceCell; // Use once_cell for safe one-time initialization
use shuttle_runtime::SecretStore;

#[derive(Debug)]
pub struct Config {
    pub github_app_id: String,
    pub github_app_pk_base64: String,
    pub github_webhook_secret: String,
    // pub github_client_secret: String,
    // pub org_token: String,
    // pub github_oauth_client_id: String,
    // pub github_oauth_client_secret: String,
    pub bot_cookie_base64: String,
    pub loggin_enabled: bool,
}

static STATIC_CONFIG: OnceCell<Config> = OnceCell::new();

pub fn init_config(store: &SecretStore) {
    let config = Config {
        github_app_id: store
            .get("GITHUB_APP_ID")
            .expect("GITHUB_APP_ID not found in secrets"),
        github_app_pk_base64: store
            .get("GITHUB_APP_PK_BASE64")
            .expect("GITHUB_APP_PK_BASE64 not found in secrets"),
        github_webhook_secret: store
            .get("GITHUB_WEBHOOK_SECRET")
            .expect("GITHUB_WEBHOOK_SECRET not found in secrets"),
        // github_client_secret: store.get("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET not found in secrets"),
        // org_token: store.get("ORG_TOKEN").expect("ORG_TOKEN not found in secrets"),
        // github_oauth_client_id: store.get("GITHUB_OAUTH_CLIENT_ID").expect("GITHUB_OAUTH_CLIENT_ID not found in secrets"),
        // github_oauth_client_secret: store.get("GITHUB_OAUTH_CLIENT_SECRET").expect("GITHUB_OAUTH_CLIENT_SECRET not found in secrets"),
        bot_cookie_base64: store
            .get("BOT_COOKIE_BASE64")
            .expect("BOT_COOKIE_BASE64 not found in secrets"),
        loggin_enabled: store.get("LOGGIN_ENABLED").unwrap_or("false".to_string()) == "true"
    };
    STATIC_CONFIG
        .set(config)
        .expect("Config has already been initialized");
}

pub fn get_config() -> &'static Config {
    STATIC_CONFIG
        .get()
        .expect("Config not initialized. Call `init_config` first.")
}

pub fn is_logging_enabled() -> bool {
    get_config().loggin_enabled
}