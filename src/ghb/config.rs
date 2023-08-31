use lazy_static::lazy_static;
use std::env;

pub struct Config {
    pub github_app_id: String,
    pub github_app_pk_base64: String,
    pub github_webhook_secret: String,
    pub github_client_secret: String,
    pub org_token: String,
    pub github_oauth_client_id: String,
    pub github_oauth_client_secret: String,
    pub bot_cookie_base64: String,
}

lazy_static! {
    pub static ref STATIC_CONFIG: Config = Config {
        github_app_id: env::var("GITHUB_APP_ID").unwrap(),
        github_app_pk_base64: env::var("GITHUB_APP_PK_BASE64").unwrap(),
        github_webhook_secret: env::var("GITHUB_WEBHOOK_SECRET").unwrap(),
        github_client_secret: env::var("GITHUB_CLIENT_SECRET").unwrap(),
        org_token: env::var("ORG_TOKEN").unwrap(),
        github_oauth_client_id: env::var("GITHUB_OAUTH_CLIENT_ID").unwrap(),
        github_oauth_client_secret: env::var("GITHUB_OAUTH_CLIENT_SECRET").unwrap(),
        bot_cookie_base64: env::var("BOT_COOKIE_BASE64").unwrap()
    };
}

pub fn get_config () -> &'static Config {
    return &STATIC_CONFIG;
}

