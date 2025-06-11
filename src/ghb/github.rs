use rouille::{ Request, Response };
use tokio::runtime::Handle as TokioHandle;
use std::io::Read;

use base64::alphabet;
use base64::engine::general_purpose::{GeneralPurpose, PAD};
use base64::engine::Engine;
use chrono::{Utc}; // Updated for specific format
use once_cell::sync::OnceCell; // For safely initializing global mutable data once
use std::sync::Mutex; // For thread-safe mutable access to global data


use crate::ghb::config::get_config;
use crate::ghb::hmac::verify_signature; // get_config is used here implicitly by global config
use crate::ghb::constants::GITHUB_API_BASE;
use crate::ghb::constants::{
    ALLOWED_ORGS,
    ALLOWED_REPOS,
    INSTALLATION_MAP,
    REPO_MAP
};

use crate::ghb::ghapi::headers::add_github_req_header;
use crate::ghb::ghapi::organisations::{
    gh_invite_user_to_org, gh_rem_user_from_org, gh_check_member
};
use crate::ghb::ghapi::private_gh::{
    pv_gh_announce_collaborator_multipart
};

static GLOBAL_INST_TOKEN: OnceCell<Mutex<String>> = OnceCell::new();
static GLOBAL_INST_TOKEN_EXP: OnceCell<Mutex<i64>> = OnceCell::new();

fn get_app_pk_from_base64() -> String {
    let engine = GeneralPurpose::new(&alphabet::STANDARD, PAD);
    let decoded = engine
        .decode(get_config().github_app_pk_base64.as_bytes())
        .unwrap();
    String::from_utf8(decoded).unwrap()
}



fn create_token() -> String {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Claims {
        aud: Option<String>,
        exp: usize,
        iat: Option<usize>,
        iss: Option<String>,
        nbf: Option<usize>,
        sub: Option<String>,
    }

    let now = Utc::now();
    let ts_60s_before = (now.timestamp() - 60).try_into().unwrap();
    let ts_5e2s_after = (now.timestamp() + 500).try_into().unwrap();

    let claims = Claims {
        exp: ts_5e2s_after,
        iat: Some(ts_60s_before),
        iss: Some(get_config().github_app_id.to_owned()),
        nbf: Some(ts_60s_before),
        sub: None,
        aud: None,
    };

    let secret = get_app_pk_from_base64();
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(secret.as_bytes()).unwrap();
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let token = jsonwebtoken::encode(&header, &claims, &key);

    if token.is_err() {
        eprintln!("token error: {:?}", token); // Use eprintln for errors
        return String::new(); // Return empty string on error
    }
    token.unwrap()
}

pub fn get_installation_token() -> String {
    if check_token_expiration() {
        // Access the token from the safely initialized global
        let token_mutex = GLOBAL_INST_TOKEN
            .get()
            .expect("GLOBAL_INST_TOKEN not initialized when expected");
        token_mutex.lock().unwrap().to_string()
    } else {
        create_installation_token(ALLOWED_ORGS[0].to_string())
    }
}

fn check_token_expiration() -> bool {
    let now = Utc::now().timestamp();

    // Safely get the Mutex. If it's not initialized, return false.
    let global_exp_mutex = match GLOBAL_INST_TOKEN_EXP.get() {
        Some(m) => m,
        None => return false,
    };

    let exp = global_exp_mutex.lock().unwrap();

    if now > (*exp - 60) {
        // Check if now is greater than 60 seconds before expiration
        return false;
    }
    true // Token is still valid
}

fn create_installation_token(for_org: String) -> String {
    let inst_id = INSTALLATION_MAP.get(for_org.as_str()).unwrap_or(&0);
    let url = format!(
        "{}/app/installations/{}/access_tokens",
        GITHUB_API_BASE, inst_id
    );
    let response = minreq::post(url);
    let send_result = add_github_req_header(&response, &create_token()).send();

    if send_result.is_err() {
        eprintln!(
            "Error sending request for installation token: {:?}",
            send_result.err()
        );
        return String::new();
    }

    let send_result = send_result.unwrap();

    if send_result.status_code != 201 {
        eprintln!(
            "Failed to get installation token, status code: {}",
            send_result.status_code
        );
        eprintln!("Response body: {}", send_result.as_str().unwrap_or(""));
        return String::new();
    }

    let body = send_result.as_str().unwrap_or("");
    let body_json: serde_json::Value = serde_json::from_str(body).unwrap_or_else(|e| {
        eprintln!("Failed to parse installation token response: {}", e);
        serde_json::Value::Null
    });

    let token_str = body_json["token"].as_str().unwrap_or("");
    let exp_str = body_json["expires_at"].as_str().unwrap_or("");

    // Parse expiration date from ISO 8601 string to timestamp
    let exp_datetime = match chrono::DateTime::parse_from_rfc3339(exp_str) {
        Ok(dt) => dt.timestamp(),
        Err(e) => {
            eprintln!("Failed to parse expiration date '{}': {}", exp_str, e);
            0 // Default to 0 on error
        }
    };

    // Safely update GLOBAL_INST_TOKEN and GLOBAL_INST_TOKEN_EXP
    let mut inst_token_guard = GLOBAL_INST_TOKEN
        .get_or_init(|| Mutex::new(String::new()))
        .lock()
        .unwrap();
    inst_token_guard.clear();
    inst_token_guard.push_str(token_str);

    let mut inst_exp_guard = GLOBAL_INST_TOKEN_EXP
        .get_or_init(|| Mutex::new(0))
        .lock()
        .unwrap();
    *inst_exp_guard = exp_datetime;

    token_str.to_string()
}



fn get_repo_from_fn(full_name: String) -> String {
    let splits: Vec<&str> = full_name.split('/').collect();
    splits.get(1).unwrap_or(&"").to_string() // Use .get() and unwrap_or for safer access
}

fn get_org_from_fn(full_name: String) -> String {
    let splits: Vec<&str> = full_name.split('/').collect();
    splits.get(0).unwrap_or(&"").to_string() // Use .get() and unwrap_or for safer access
}

fn check_repo_and_org_allowed(input: &Result<serde_json::Value, String>) -> bool {
    let input_value = input.as_ref().unwrap(); // Assuming input is always Ok here
    let full_name = input_value["repository"]["full_name"]
        .as_str()
        .unwrap_or("");
    let repo: String = get_repo_from_fn(full_name.to_string());
    let org: String = get_org_from_fn(full_name.to_string());

    if !ALLOWED_ORGS.contains(&org.as_str()) {
        return false;
    }
    if !ALLOWED_REPOS.contains(&repo.as_str()) {
        return false;
    }
    true
}

pub fn check_auth() -> bool {
    let url = format!("{}/app", GITHUB_API_BASE);

    let response = minreq::get(url);
    let send_result = add_github_req_header(&response, &create_token()).send();

    let send_result = match send_result {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Auth check request failed status: {:?}", e);
            return false;
        }
    };

    if send_result.status_code != 200 {
        eprintln!(
            "Auth check failed, status code: {}",
            send_result.status_code
        );
        eprintln!("Response body: {}", send_result.as_str().unwrap_or(""));
        return false;
    }
    true
}



fn get_event_type(input: &Result<serde_json::Value, String>) -> String {
    let input_value = input.as_ref().unwrap();
    let action = input_value["action"].as_str().unwrap_or("");
    let starred_url = input_value["sender"]["starred_url"].as_str().unwrap_or("");

    if action == "created" && !starred_url.is_empty() {
        return "star_created".to_string();
    }
    if action == "deleted" && !starred_url.is_empty() {
        return "star_deleted".to_string();
    }
    String::new()
}

// Handler functions need to accept a `&serde_json::Value` if `Ok` is already unwrapped
fn register_event_handler(
    event_type: &str,
    handler: fn(input: &serde_json::Value),
    input_value: &serde_json::Value,
) {
    const AVAILABLE_EVENTS: [&str; 2] = ["star_created", "star_deleted"];
    if AVAILABLE_EVENTS.contains(&event_type) {
        // Need to clone the input_value for get_event_type as it expects &Result<V,S>
        // This is a bit awkward. It's better to pass `serde_json::Value` directly.
        let temp_input_result = Ok(input_value.clone());
        let req_event_type: &str = &get_event_type(&temp_input_result);

        if req_event_type == event_type {
            handler(input_value); // Pass the raw Value
        }
    } else {
        eprintln!("Event type '{}' not available", event_type);
    }
}

fn handle_star_created(input: &serde_json::Value) {
    // Changed signature
    let full_name = input["repository"]["full_name"].as_str().unwrap_or("");
    let repo: String = get_repo_from_fn(full_name.to_string());
    let user_id = input["sender"]["id"].as_i64().unwrap_or_default();
    let user = input["sender"]["login"].as_str().unwrap_or("");
    let asoc_repo = REPO_MAP
        .get(repo.as_str())
        .unwrap_or(&String::new())
        .to_string(); // Use String::new()
    let is_member = gh_check_member(ALLOWED_ORGS[0], user);
    if is_member {
        println!(
            "User {} is member in org {}, returning",
            user, asoc_repo
        );
        return;
    }
    println!("User id {} is not member in org {}", user_id, asoc_repo);
    let is_inv_ok = gh_invite_user_to_org(ALLOWED_ORGS[0], user_id);
    if is_inv_ok {
        println!("User {} invited to repo {}", user, asoc_repo);
        pv_gh_announce_collaborator_multipart(user.to_string());
    } else {
        eprintln!("Failed to invite user {} to repo {}", user, asoc_repo);
    }
}

fn handle_star_deleted(input: &serde_json::Value) {
    // Changed signature
    let full_name = input["repository"]["full_name"].as_str().unwrap_or("");
    let repo: String = get_repo_from_fn(full_name.to_string());
    let user = input["sender"]["login"].as_str().unwrap_or("");
    let asoc_repo = REPO_MAP
        .get(repo.as_str())
        .unwrap_or(&String::new())
        .to_string(); // Use String::new()
    let is_member = gh_check_member(ALLOWED_ORGS[0], user);
    if !is_member {
        println!(
            "User {} is not a member in org {}, returning",
            user, asoc_repo
        );
        return;
    }
    let is_del_ok = gh_rem_user_from_org(ALLOWED_ORGS[0], user);
    if is_del_ok {
        println!("User {} deleted from repo {}", user, asoc_repo);
    } else {
        eprintln!("Failed to delete user {} from repo {}", user, asoc_repo);
    }
}

pub fn handle_hook(request: &Request,  runtime_handle: TokioHandle) -> Response {
    let mut data = request
        .data()
        .expect("Oops, body already retrieved, problem in the server");

    let mut buf = Vec::new();

    match data.read_to_end(&mut buf) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to read request body: {}", e);
            return Response::text("Failed to read body").with_status_code(500);
        }
    };

    let mut map = serde_json::Map::new();
    let input_value: serde_json::Value = match serde_json::from_slice(&buf) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to parse request body as JSON: {}", e);
            map.insert(
                "status".to_string(),
                serde_json::Value::String("error".to_string()),
            );
            map.insert(
                "message".to_string(),
                serde_json::Value::String(format!("Invalid JSON body: {}", e)),
            );
            return Response::json(&map).with_status_code(400);
        }
    };

    if input_value.is_null() {
        return Response::json({
            map.insert(
                "status".to_string(),
                serde_json::Value::String("error".to_string()),
            );
            map.insert(
                "message".to_string(),
                serde_json::Value::String("No body provided or body is null".to_string()),
            );
            &map
        })
        .with_status_code(400);
    }

    let signature = request
        .header("X-Hub-Signature-256")
        .unwrap_or("")
        .to_string();
    let secret = get_config().github_webhook_secret.to_string(); // Assuming get_config() is safe after init_config()

    let is_valid = verify_signature(buf, &signature, &secret);

    if !is_valid {
        return Response::json({
            map.insert(
                "status".to_string(),
                serde_json::Value::String("error".to_string()),
            );
            map.insert(
                "message".to_string(),
                serde_json::Value::String(
                    "Invalid hmac signature, check webhook secret".to_string(),
                ),
            );
            &map
        })
        .with_status_code(400);
    }

    let is_allowed = check_repo_and_org_allowed(&Ok(input_value.clone())); // Pass a clone for the check

    if !is_allowed {
        return Response::json({
            map.insert(
                "status".to_string(),
                serde_json::Value::String("error".to_string()),
            );
            map.insert(
                "message".to_string(),
                serde_json::Value::String("Not allowed repo / org".to_string()),
            );
            &map
        })
        .with_status_code(400);
    }

    // Get a handle to the current Tokio runtime and spawn the async tasks
    let input_value_clone = input_value.clone(); // Clone for the spawned task
    runtime_handle.spawn(async move {
        // Pass the cloned `serde_json::Value` directly to handlers
        register_event_handler("star_created", handle_star_created, &input_value_clone);
        register_event_handler("star_deleted", handle_star_deleted, &input_value_clone);
    });

    Response::json({
        map.insert(
            "status".to_string(),
            serde_json::Value::String("ok".to_string()),
        );
        map.insert(
            "message".to_string(),
            serde_json::Value::String("Webhook processed".to_string()),
        ); // More descriptive message
        &map
    })
    .with_status_code(200)
}
