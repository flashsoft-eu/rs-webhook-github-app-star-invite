use crate::ghb::constants::GITHUB_API_BASE;
use crate::ghb::ghapi::headers::add_github_req_header;
use crate::ghb::github::get_installation_token;

#[allow(dead_code)]
pub fn gh_invite_collaborator(org: &str, repo: &str, user: &str) -> bool {
    let url = format!(
        "{}/repos/{}/{}/collaborators/{}",
        GITHUB_API_BASE, org, repo, user
    );

    let response = minreq::put(url);
    let send_result = add_github_req_header(&response, &get_installation_token())
        .with_body("{\"permission\":\"pull\"}")
        .send();

    if send_result.is_err() {
        eprintln!(
            "Invite collaborator request failed: {:?}",
            send_result.err()
        );
        return false;
    }

    let send_result = send_result.unwrap();

    if [204, 201].contains(&send_result.status_code) {
        return true;
    }
    eprintln!(
        "Failed to invite collaborator, status code: {}",
        send_result.status_code
    );
    false
}

#[allow(dead_code)]
pub fn gh_delete_collaborator(org: &str, repo: &str, user: &str) -> bool {
    let url = format!(
        "{}/repos/{}/{}/collaborators/{}",
        GITHUB_API_BASE, org, repo, user
    );

    let response = minreq::delete(url);
    let send_result = add_github_req_header(&response, &get_installation_token()).send();

    if send_result.is_err() {
        eprintln!(
            "Delete collaborator request failed: {:?}",
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
        "Failed to delete collaborator, status code: {}",
        send_result.status_code
    );
    false
}

#[allow(dead_code)]
pub fn gh_check_colaborator(org: &str, repo: &str, user: &str) -> bool {
    let url = format!(
        "{}/repos/{}/{}/collaborators/{}",
        GITHUB_API_BASE, org, repo, user
    );

    let response = minreq::get(url);
    let send_result = add_github_req_header(&response, &get_installation_token()).send();

    if send_result.is_err() {
        eprintln!("Collaborator check request failed: {:?}", send_result.err());
        return false;
    }

    let send_result = send_result.unwrap();

    if send_result.status_code != 204 {
        // 204 No Content indicates user is a collaborator
        return false;
    }
    true
}