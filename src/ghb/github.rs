
use rouille::Request;
use rouille::Response;

use chrono::Utc;

use base64::alphabet;
use base64::engine::Engine;
use base64::engine::general_purpose::{GeneralPurpose, PAD};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {

    static ref ALLOWED_ORGS: [&'static str; 2] = ["flashsoft-eu", "andrei0x309"];

    static ref ALLOWED_REPOS: [&'static str; 1] = ["deno-slack-api-backup-preview"];

    static ref REPO_MAP: HashMap<&'static str, String> = {
        let mut map = HashMap::new();
        map.insert("deno-slack-api-backup-preview", String::from("deno-slack-user-api"));
        map
    };

    static ref INSTALLATION_MAP: HashMap<&'static str, i64> = {
        let mut map = HashMap::new();
        map.insert("flashsoft-eu", 40959841);
        map.insert("andrei0x309", 40959837);
        map
    };
}


use crate::ghb::config::get_config;

static GITHUB_API_BASE : &str = "https://api.github.com";

fn get_app_pk_from_base64 () -> String {
    let engine = GeneralPurpose::new(&alphabet::STANDARD, PAD);
    let decoded = engine.decode(get_config().github_app_pk_base64.as_bytes()).unwrap();
    let app_pk = String::from_utf8(decoded).unwrap();
    return app_pk;
}

fn get_user_cookie_from_base64 () -> String {
    let engine = GeneralPurpose::new(&alphabet::STANDARD, PAD);
    let decoded = engine.decode(get_config().bot_cookie_base64.as_bytes()).unwrap();
    let cookie = String::from_utf8(decoded).unwrap();
    return cookie;
}


fn create_token () -> String {

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Claims {
    aud: Option<String>,         // Optional. Audience
    exp: usize,          // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: Option<usize>,          // Optional. Issued at (as UTC timestamp)
    iss: Option<String>,         // Optional. Issuer
    nbf: Option<usize>,          // Optional. Not Before (as UTC timestamp)
    sub: Option<String>,         // Optional. Subject (whom token refers to)
}

    let now = Utc::now();
    let ts_60s_before = (now.timestamp() - 60).try_into().unwrap();
    let ts_6e2s_after = (now.timestamp() + 600).try_into().unwrap();

    let claims = Claims {
        exp: ts_6e2s_after,
        iat: Some(ts_60s_before),
        iss: Some(get_config().github_app_id.to_owned()),
        nbf: Some(ts_60s_before),
        sub: None,
        aud: None
    };

    let secret = get_app_pk_from_base64();
    let key = jsonwebtoken::EncodingKey::from_rsa_pem( secret.as_bytes() ).unwrap();
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let token = jsonwebtoken::encode(&header, &claims, &key);
    
    if token.is_err() {
        println!("token error: {:?}", token);
        return "".to_string();
    }
    return token.unwrap();
}


fn create_installation_token (for_org: String) -> String {
    let inst_id = INSTALLATION_MAP.get(&for_org.as_str()).unwrap_or(&0);
    let url: String = format!("{}/app/installations/{}/access_tokens", GITHUB_API_BASE, inst_id);
    let response: minreq::Request = minreq::post(url);
    let send_result = add_github_req_header(&response, &create_token()).send();

    if send_result.is_err() {
        return "".to_string();
    }


    let send_result = send_result.unwrap();

    if send_result.status_code != 201 {
        return "".to_string();
    }

    let body = send_result.as_str().unwrap_or("");

    let body: serde_json::Value = serde_json::from_str(body).unwrap();

    let token = body["token"].as_str().unwrap_or("");

    return token.to_string();
}


fn add_github_req_header<'a, 'b>(response: &'a minreq::Request, token: &'b str) -> minreq::Request {
    let mut modified_response =  response.clone();
    modified_response = modified_response.with_header("Accept", "application/vnd.github+json");
    modified_response = modified_response.with_header("Authorization", format!("Bearer {}", token));
    modified_response = modified_response.with_header("X-GitHub-Api-Version", "2022-11-28");
    modified_response = modified_response.with_header("User-Agent", format!("Rust ghb/{}", env!("CARGO_PKG_VERSION")));
    modified_response
}

fn get_repo_from_fn (full_name: String) -> String {
    let splits: Vec<&str> = full_name.split('/').collect();
    return splits.as_slice()[1].to_string();
}

fn get_org_from_fn (full_name: String) -> String {
    let splits: Vec<&str> = full_name.split('/').collect();
    return splits.as_slice()[0].to_string();
}
  

fn check_repo_and_org_allowed (input: &Result<serde_json::Value, String> ) -> bool {
    let input =  input.as_ref().unwrap();
    let full_name = input["repository"]["full_name"].as_str().unwrap_or("");
    let repo: String = get_repo_from_fn(full_name.to_string());
    let org: String = get_org_from_fn(full_name.to_string());


    if !ALLOWED_ORGS.contains(&&org.as_str()) {
        return false;
    }
    if !ALLOWED_REPOS.contains(&&repo.as_str()) {
        return false;
    }
    return true;
}


pub fn check_auth() -> bool {
 
let url = format!("{}/app", GITHUB_API_BASE);

let response: minreq::Request = minreq::get(url);
let send_result = add_github_req_header(&response, &create_token()).send();

// println!("send_result: {:?}", send_result);

if send_result.is_err() {
    return false;
}

let send_result = send_result.unwrap();

if send_result.status_code != 200 {
    return false;
}

return true;

}

fn gh_check_colaborator (org: &str, repo: &str, user: &str) -> bool {
//     curl -L \
//   -H "Accept: application/vnd.github+json" \
//   -H "Authorization: Bearer <YOUR-TOKEN>" \
//   -H "X-GitHub-Api-Version: 2022-11-28" \
//   https://api.github.com/repos/OWNER/REPO/collaborators/USERNAME

    let url = format!("{}/repos/{}/{}/collaborators/{}", GITHUB_API_BASE, org, repo, user);

    let response: minreq::Request = minreq::get(url);
    let send_result = add_github_req_header(&response, &create_installation_token(
        ALLOWED_ORGS[0].to_string()
    )).send();

    // println!("send_result: {:?}", send_result);

    if send_result.is_err() {
        return false;
    }

    let send_result = send_result.unwrap();

    if send_result.status_code != 204 {
        return false;
    }

    return true;
}

fn gh_invite_collaborator (org: &str, repo: &str, user: &str) -> bool {
 
    let url = format!("{}/repos/{}/{}/collaborators/{}", GITHUB_API_BASE, org, repo, user);

    let response: minreq::Request = minreq::put(url);
    let send_result = add_github_req_header(&response, &create_installation_token(
        ALLOWED_ORGS[0].to_string()
    )).with_body("{\"permission\":\"pull\"}").send();

    // println!("send_result: {:?}", std::str::from_utf8(&send_result.as_mut().unwrap().as_bytes()));

    // println!("send_result: {:?}", send_result);

    if send_result.is_err() {
        return false;
    }

    let send_result = send_result.unwrap();
    

    if [204, 201].contains(&send_result.status_code) {
        return true;
    }

    return false;

}

fn gh_delete_collaborator (org: &str, repo: &str, user: &str) -> bool {
     
        let url = format!("{}/repos/{}/{}/collaborators/{}", GITHUB_API_BASE, org, repo, user);
    
        let response: minreq::Request = minreq::delete(url);
        let send_result = add_github_req_header(&response, &create_installation_token(
            ALLOWED_ORGS[0].to_string()
        )).send();
    
        if send_result.is_err() {
            return false;
        }
    
        let send_result = send_result.unwrap();
        
    
        if [204, 201].contains(&send_result.status_code) {
            return true;
        }
    
        return false;

}

fn pv_gh_user_header (response: &minreq::Request) -> minreq::Request {
    
    let cookie =  get_user_cookie_from_base64();

    // println!("cookie: {:?}", cookie);
    
    let mut modified_response = response.clone();
    modified_response = modified_response.with_header("Accept", "text/html");
    modified_response = modified_response.with_header("Content-Type", "text/html");
    modified_response = modified_response.with_header("Cookie", cookie);
    modified_response = modified_response.with_header("User-Agent", "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Mobile Safari/537.36");
    modified_response = modified_response.with_header("Origin", "https://github.com");
    modified_response = modified_response.with_header("Referer", format!("https://github.com/orgs/{}/discussions/1", ALLOWED_ORGS[0]));
    modified_response = modified_response.with_header("DNT", "1");
    modified_response = modified_response.with_header("Pragma", "no-cache");
    modified_response = modified_response.with_header("Sec-Ch-Ua", "\"Chromium\";v=\"116\", \"Not)A;Brand\";v=\"24\", \"Brave\";v=\"116\"");
    modified_response = modified_response.with_header("Sec-Ch-Ua-Mobile", "?1");
    modified_response = modified_response.with_header("Sec-Ch-Ua-Platform", "\"Android\"");
    modified_response = modified_response.with_header("Sec-Fetch-Dest", "empty");
    modified_response = modified_response.with_header("Sec-Fetch-Mode", "cors");
    modified_response = modified_response.with_header("Sec-Fetch-Site", "same-origin");
    modified_response = modified_response.with_header("Sec-Gpc", "1");

    modified_response
}


fn pv_gh_get_crsf_token () -> String {

    let url = format!("https://github.com/orgs/{}/discussions/1", ALLOWED_ORGS[0]);

    // println!("url: {:?}", url);

    let req: minreq::Request = minreq::get(url);
    let req = pv_gh_user_header(&req);
    let send_result = req.send();
    
    if send_result.is_err() {
        return "".to_string();
    }

    let send_result = send_result.unwrap();
    let body = send_result.as_str().unwrap_or("");

 
    let re = regex::Regex::new("/discussions/1/comments.*?authenticity.*?value=(?:\"|')(.*?)(?:\"|')").unwrap();


    let token = re.captures(body);

    if token.is_none() {
        return "".to_string();
    }
 
    let token =  token.unwrap().get(1).map_or("", |m| m.as_str());

    // println!("token: {:?}", token);

    return token.to_string();
}

fn pv_gh_announce_collaborator ( user: String, repo: String ) -> bool {
 
    let token = pv_gh_get_crsf_token();

    if token == "" {
        println! ("crfs token not found");
        return false;
    }

    let url = format!("https://github.com/{}/discussions-host/discussions/1/comments", ALLOWED_ORGS[0]);
    
    let req: minreq::Request = minreq::post(url);
    let mut req = pv_gh_user_header(&req);
    req = req.with_header("Content-Type", "application/x-www-form-urlencoded");
    
    let date = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let message = format!("[Automated] @{} has been invited to the repo {}. Invitation was sent at {}",  user, repo, date);

    let body = format!("timestamp={}&comment[body]={}&authenticity_token={}&required_field_609c=&timestamp_secret=&saved_reply_id=&path=&line=&start_line=&preview_side=&preview_start_side=&start_commit_oid=&end_commit_oid=&base_commit_oid=&comment_id=", Utc::now().timestamp(), message, token);
    let send_result = req.with_body(body).send();

    if send_result.is_err() {
        return false;
    }

    return true;
}


fn get_event_type(input: &Result<serde_json::Value, String>) -> String {
    let input =  input.as_ref().unwrap();
    let action = input["action"].as_str().unwrap_or("");
    let starred_url = input["sender"]["starred_url"].as_str().unwrap_or("");

    if action == "created" && starred_url != "" {
        return "star_created".to_string();
    }
    if action == "deleted" && starred_url != "" {
        return "star_deleted".to_string();
    }
    return "".to_string();
}


fn register_event_handler(event_type: &str, handler: fn(input: &Result<serde_json::Value, String>) -> (), input: &Result<serde_json::Value, String>) {
    const AVAILABLE_EVENTS: [&str; 2] = ["star_created", "star_deleted"];
    if AVAILABLE_EVENTS.contains(&event_type) {
        let req_event_type: &str = &get_event_type(&input);
        if req_event_type == event_type {
            handler(&input);
        }
    } else {
        println!("Event type not available");
    }
}

fn handle_star_created (input: &Result<serde_json::Value, String>) {
    let full_name = input.as_ref().unwrap()["repository"]["full_name"].as_str().unwrap_or("");
    let repo: String = get_repo_from_fn(full_name.to_string());
    let user = input.as_ref().unwrap()["sender"]["login"].as_str().unwrap_or("");
    let asoc_repo = REPO_MAP.get(&repo.as_str()).unwrap_or(&String::from("")).to_string();
    let is_colab = gh_check_colaborator( ALLOWED_ORGS[0], &asoc_repo, user );
    if is_colab {
        println!("User {} is colaborator on repo {}, returning", user, asoc_repo);
        return;
    }
    let is_inv_ok = gh_invite_collaborator( ALLOWED_ORGS[0], &asoc_repo, user );
    if is_inv_ok {
        println!("User {} invited to repo {}", user, asoc_repo);
        pv_gh_announce_collaborator(user.to_string(), asoc_repo);
    } 

}

fn handle_star_deleted (input: &Result<serde_json::Value, String>) {
    let full_name = input.as_ref().unwrap()["repository"]["full_name"].as_str().unwrap_or("");
    let repo: String = get_repo_from_fn(full_name.to_string());
    let user = input.as_ref().unwrap()["sender"]["login"].as_str().unwrap_or("");
    let asoc_repo = REPO_MAP.get(&repo.as_str()).unwrap_or(&String::from("")).to_string();
    let is_colab = gh_check_colaborator( ALLOWED_ORGS[0], &asoc_repo, user );
    if !is_colab {
        println!("User {} is not colaborator on repo {}, returning", user, asoc_repo);
        return;
    }
    let is_del_ok = gh_delete_collaborator( ALLOWED_ORGS[0], &asoc_repo, user );
    if is_del_ok {
        println!("User {} deleted from repo {}", user, asoc_repo);
    }
}


pub fn handle_hook(request : &Request) -> Response {

    let mut map = serde_json::Map::new();
    let input = rouille::input::json_input::<serde_json::Value>(request).unwrap();

    if input.is_null() {
        return Response::json({
            map.insert("status".to_string(), serde_json::Value::String("error".to_string()));
            map.insert("message".to_string(), serde_json::Value::String("No body provided".to_string()));
            &map}).with_status_code(400);
    }

    let ok_input = Ok(input);

    let is_allowed = check_repo_and_org_allowed(&ok_input);

    if !is_allowed {
        return Response::json({
            map.insert("status".to_string(), serde_json::Value::String("error".to_string()));
            map.insert("message".to_string(), serde_json::Value::String("Not allowed repo / org".to_string()));
            &map}).with_status_code(400);
    }


    register_event_handler("star_created", handle_star_created, &ok_input);
    register_event_handler("star_deleted", handle_star_deleted, &ok_input);

    Response::json({
        map.insert("status".to_string(), serde_json::Value::String("ok".to_string()));
        map.insert("message".to_string(), serde_json::Value::String("ok".to_string()));
        &map}).with_status_code(200)
}
