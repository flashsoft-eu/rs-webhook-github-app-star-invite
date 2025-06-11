use lazy_static::lazy_static; // Still used for truly static, immutable data
use std::collections::HashMap;


pub static GITHUB_API_BASE: &str = "https://api.github.com";

lazy_static! {
    pub static ref ALLOWED_ORGS: [&'static str; 1] = ["flashsoft-eu"];
    pub static ref ALLOWED_REPOS: [&'static str; 1] = ["access-to-private-repos"];
    pub static ref REPO_MAP: HashMap<&'static str, String> = {
    let mut map = HashMap::new();
        map.insert(
            "access-to-private-repos",
            String::from("access-to-private-repos"),
        );
        map
    };
    pub static ref INSTALLATION_MAP: HashMap<&'static str, i64> = {
        let mut map = HashMap::new();
        map.insert("flashsoft-eu", 40959841);
        // map.insert("andrei0x309", 40959837);
        map
    };
}