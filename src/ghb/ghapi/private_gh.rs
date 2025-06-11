use rand::Rng;
use rouille::url::form_urlencoded;
use crate::ghb::config::{get_config, is_logging_enabled};
use base64::alphabet;
use base64::engine::general_purpose::{GeneralPurpose, PAD};
use base64::engine::Engine;
use chrono::{SecondsFormat, Utc}; // Updated for specific format
use uuid::Uuid;

use crate::ghb::constants::{
    ALLOWED_ORGS,
};

// Helper function to create a single form part
fn create_form_part(boundary: &str, name: &str, value: &str) -> String {
    format!(
        "--{}\r\nContent-Disposition: form-data; name=\"{}\"\r\n\r\n{}\r\n",
        boundary, name, value
    )
}


fn get_user_cookie_from_base64() -> String {
    let engine = GeneralPurpose::new(&alphabet::STANDARD, PAD);
    let decoded = engine
        .decode(get_config().bot_cookie_base64.as_bytes())
        .unwrap();
    String::from_utf8(decoded).unwrap()
}


fn pv_gh_user_header(response: &minreq::Request) -> minreq::Request {
    let cookie =  get_user_cookie_from_base64();

    let mut modified_response = response.clone();
    modified_response = modified_response.with_header("Accept", "text/html");
    modified_response = modified_response.with_header("Content-Type", "text/html");
    modified_response = modified_response.with_header("Cookie", cookie);
    modified_response = modified_response.with_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36");
    modified_response = modified_response.with_header("Origin", "https://github.com");
    modified_response = modified_response.with_header(
        "Referer",
        format!("https://github.com/orgs/{}/discussions/1", ALLOWED_ORGS[0]),
    );
 
    modified_response
}

fn pv_gh_get_crsf_token() -> (String, String, String, String) {
    let url = format!("https://github.com/orgs/{}/discussions/1", ALLOWED_ORGS[0]);

    let req = minreq::get(url);
    let req = pv_gh_user_header(&req);
    let send_result = req.send();

    if send_result.is_err() {
        eprintln!("Failed to get CRSF token: {:?}", send_result.err());
        return (String::new(), String::new(), String::new(), String::new());
    }

    let send_result = send_result.unwrap();
    let body = send_result.as_str().unwrap_or("");

    // Regex to extract authenticity token
    // This regex looks for `/discussions/1/comments` followed by anything,
    // then `authenticity`, then `value=` and captures the content inside quotes.
    let re =
        regex::Regex::new(r#"(?s)discussions/1/comments.*?authenticity_token.*?value=["']([^"']*)["']"#)
            .expect("Failed to compile regex");

    let token_capture = re.captures(body);
    let mut token = String::new();

    if token_capture.is_none() {
        eprintln!("authenticity_token regex match failed on body");
    } else {
        token = token_capture
        .unwrap()
        .get(1)
        .map_or(String::new(), |m| m.as_str().to_string())
    }

    let re = regex::Regex::new(r#"(?s)name.*?(required_field[^"']*)["']"#)
    .expect("Failed to compile regex");

    let required_fieldcapture = re.captures(body);
    let mut required_field = String::new();

    if required_fieldcapture.is_none() {
       eprintln!("required_field regex match failed on body");
    }
    else {
        required_field = required_fieldcapture
        .unwrap()
        .get(1)
        .map_or(String::new(), |m| m.as_str().to_string())
    }

    let re = regex::Regex::new(r#"(?s)name.*?timestamp["'].*?value=["']([^"']*)["']"#)
    .expect("Failed to compile regex");

    let timestamp_capture = re.captures(body);
    let mut timestamp = String::new();
    if timestamp_capture.is_none() {
        eprintln!("timestamp regex match failed on body");
    } else {
        timestamp = timestamp_capture
        .unwrap()
        .get(1)
        .map_or(String::new(), |m| m.as_str().to_string())
    }

    let re = regex::Regex::new(r#"(?s)name.*?timestamp_secret.*?value=["']([^"']*)["']"#)
    .expect("Failed to compile regex");

    let timestamp_secret_capture = re.captures(body);

    let mut timestamp_secret = String::new();
    if timestamp_secret_capture.is_none() {
       eprintln!("timestamp_secret regex match failed on body");
    } else {
        timestamp_secret = timestamp_secret_capture
        .unwrap()
        .get(1)
        .map_or(String::new(), |m| m.as_str().to_string())
    }

    return (token, required_field, timestamp, timestamp_secret);
 
}


#[allow(dead_code)]
pub fn pv_gh_announce_collaborator_multipart(user: String) -> bool {
    let data = pv_gh_get_crsf_token();
    let auth_token = data.0;
    let required_field = data.1;
    let timestamp = data.2;
    let timestamp_secret = data.3;

    if auth_token.is_empty() || required_field.is_empty() || timestamp.is_empty() || timestamp_secret.is_empty() {
        eprintln!("Failed to get from data: {:?}", (auth_token, required_field, timestamp, timestamp_secret));
        return false;
    }

    let url = format!(
        "https://github.com/{}/access-to-private-repos/discussions/1/comments",
        ALLOWED_ORGS[0]
    );

    let req = minreq::post(url);

    let boundary_bytes: [u8; 4] = rand::thread_rng().r#gen();
    let boundary_random = hex::encode(boundary_bytes);
    let boundary = format!("----WebKitFormBoundary{}", boundary_random);

    let mut req = pv_gh_user_header(&req);
    req = req.with_header("Content-Type", format!("multipart/form-data; boundary={}", boundary));
    req = req.with_header("Accept", "application/json");

    let date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true); // Use SecondsFormat
    let message = format!("```
User https://github.com/{} has been invited to join the organization.

Invitation issued on {}.
", user, date
    );
 
    let fields: Vec<(&str, &str)> = vec![
        ("authenticity_token", &auth_token),
        (&required_field, ""),
        ("timestamp", &timestamp),
        ("timestamp_secret", &timestamp_secret),
        ("saved_reply_id", ""),
        ("comment[body]", &message),
        ("path", ""),
        ("line", ""),
        ("start_line", ""),
        ("start_commit_oid", ""),
        ("end_commit_oid", ""),
        ("base_commit_oid", ""),
        ("comment_id", ""),
        // Add other fixed fields here if needed
    ];

    let mut form_parts = Vec::new();
    for (name, value) in fields {
        form_parts.push(create_form_part(&boundary, name, value));
    }
 
    let closing_boundary = format!("--{}--\r\n", boundary);
    form_parts.push(closing_boundary);
    

        let request_body = form_parts.join("");

    if is_logging_enabled() {

    println!("\n--- Constructed Request Body ---");
    println!("{}", request_body);
    println!("--------------------------------\n");

    }
    
    let content_length = request_body.len();
    req = req.with_header("Content-Length", content_length.to_string());
    let nounce_uuid = Uuid::new_v4().to_string();
    let nounce_string = format!("v2:{}", nounce_uuid);
    req = req.with_header("x-fetch-nonce", nounce_string);
    req = req.with_header("x-github-client-version", "4fec336a99e62ef8333fc10589e4bb3d9b666b06");
    req = req.with_header("x-requested-with", "XMLHttpRequest");

    let send_result = req.with_body(request_body).send();

    let is_error = send_result.is_err();

    if is_error {
        eprintln!("Failed to announce collaborator: {:?}", send_result.err());
        return false;
    }


    let unwraped_result = send_result.unwrap();
    let status_code = unwraped_result.status_code;


    if status_code == 200 || status_code == 201 {
        true
    } else {
        eprintln!(
            "Failed to announce collaborator, status code: {}",
            status_code
        );
        false
    }
}

#[allow(dead_code)]
pub fn pv_gh_announce_collaborator_urlencoded(user: String) -> bool {
    let data = pv_gh_get_crsf_token();
    let auth_token = data.0;
    let required_field = data.1;
    let timestamp = data.2;
    let timestamp_secret = data.3;

    if auth_token.is_empty() || required_field.is_empty() || timestamp.is_empty() || timestamp_secret.is_empty() {
        eprintln!("Failed to get from data: {:?}", (auth_token, required_field, timestamp, timestamp_secret));
        return false;
    }

    let url = format!(
        "https://github.com/{}/access-to-private-repos/discussions/1/comments",
        ALLOWED_ORGS[0]
    );

    let req = minreq::post(url);

    let mut req = pv_gh_user_header(&req);
    req = req.with_header("Content-Type", "application/x-www-form-urlencoded");
    req = req.with_header("Accept", "*/*");
    req = req.with_header("accept-language", "en-US,en;q=0.9");
    req = req.with_header("cache-control", "no-cache");


    let date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true); // Use SecondsFormat
    let message = format!("```
User {} has been invited to join the organization.

Invitation issued on {}.
", user, date
    );

    // URL-encode the message body if it contains special characters
    let encoded_message = form_urlencoded::byte_serialize(message.as_bytes()).collect::<String>();

    // let body = format!("authenticity_token={}&{}=&timestamp={}&timestamp_secret={}&saved_reply_id=&comment[body]={}&path=&line=&start_line=&preview_side=&preview_start_side=&start_commit_oid=&end_commit_oid=&base_commit_oid=&comment_id=", auth_token, required_field, timestamp, timestamp_secret, encoded_message);
    
    let fields: Vec<(&str, &str)> = vec![
        ("authenticity_token", &auth_token),
        (&required_field, ""),
        ("timestamp", &timestamp),
        ("timestamp_secret", &timestamp_secret),
        ("saved_reply_id", ""),
        ("saved-reply-filter-field", ""),
        ("comment[body]", &encoded_message),
        ("path", ""),
        ("line", ""),
        ("start_line", ""),
        ("start_commit_oid", ""),
        ("end_commit_oid", ""),
        ("base_commit_oid", ""),
        ("comment_id", ""),
    ];


    let mut form_parts = Vec::new();
    for (name, value) in fields {
        form_parts.push(String::from(format!("{}={}", name, value)));
    }
 
    let request_body = form_parts.join("&");

    if is_logging_enabled() {

    println!("\n--- Request Headers ---");
    println!("{:?}", req);
    println!("--------------------------------\n");

    println!("\n--- Constructed URL-Encoded Request Body ---");
    println!("{}", request_body);
    println!("--------------------------------\n");

    }
    
    let send_result = req.with_max_redirects(30).with_body(request_body).send();

    let is_error = send_result.is_err();

    if is_error {
        eprintln!("Failed to announce collaborator: {:?}", send_result.err());
        return false;
    }


    let unwraped_result = send_result.unwrap();
    let status_code = unwraped_result.status_code;


    if status_code == 200 || status_code == 201 {
        true
    } else {
        eprintln!(
            "Failed to announce collaborator, status code: {}",
            status_code
        );
        false
    }
}