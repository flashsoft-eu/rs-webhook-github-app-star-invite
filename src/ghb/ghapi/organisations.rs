use crate::ghb::constants::GITHUB_API_BASE;
use crate::ghb::ghapi::headers::add_github_req_header;
use crate::ghb::github::get_installation_token;

pub fn gh_invite_user_to_org(org: &str, invitee_id : i64) -> bool {
    let url = format!(
        "{}/orgs/{}/invitations",
        GITHUB_API_BASE, org
    );

    println!("Inviting user {}, to org {}", invitee_id, org);

    let response = minreq::post(url);
    let send_result = add_github_req_header(&response, &get_installation_token())
        .with_body(format!("{{\"invitee_id\":{}}}", invitee_id))
        .send();

    if send_result.is_err() {
        let error_request = send_result.unwrap();
        eprintln!(
            "Invite user to org request failed: {:?}",
             error_request.status_code
        );
        eprintln!("Response body: {}", error_request.as_str().unwrap_or(""));
        return false;
    }

    let send_result = send_result.unwrap();

    if [204, 201].contains(&send_result.status_code) {
        return true;
    }
    eprintln!(
        "Failed to invite user to org, status code: {}",
        send_result.status_code
    );
    eprintln!("Response body: {}", send_result.as_str().unwrap_or(""));
    false
}

pub fn gh_rem_user_from_org(org: &str, user: &str) -> bool {
    let url = format!(
        "{}/orgs/{}/members/{}",
        GITHUB_API_BASE, org, user
    );

    let response = minreq::delete(url);
    let send_result = add_github_req_header(&response, &get_installation_token()).send();

    if send_result.is_err() {
        eprintln!(
            "Removing user from org failed: {:?}",
            send_result.err()
        );
        return false;
    }

    let send_result = send_result.unwrap();

    if send_result.status_code == 204 {
        // 204 No Content indicates successful deletion
        return true;
    }
    eprintln!(
        "Failed to remove user from org, status code: {}",
        send_result.status_code
    );
    false
}

pub fn gh_check_member(org: &str, user: &str) -> bool {
    let url = format!(
        "{}/orgs/{}/members/{}",
        GITHUB_API_BASE, org, user
    );

    let response = minreq::get(url);
    let send_result = add_github_req_header(&response, &get_installation_token()).send();

    if send_result.is_err() {
        eprintln!("Member check request failed: {:?}", send_result.err());
        return false;
    }

    let send_result = send_result.unwrap();

    if send_result.status_code != 204 {
        // 204 No Content indicates user is a collaborator
        return false;
    }
    true
}