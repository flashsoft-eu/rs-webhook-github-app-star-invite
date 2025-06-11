pub fn add_github_req_header<'a, 'b>(response: &'a minreq::Request, token: &'b str) -> minreq::Request {
    let mut modified_response = response.clone();
    modified_response = modified_response.with_header("Accept", "application/vnd.github+json");
    modified_response = modified_response.with_header("Authorization", format!("Bearer {}", token));
    modified_response = modified_response.with_header("X-GitHub-Api-Version", "2022-11-28");
    modified_response = modified_response.with_header(
        "User-Agent",
        format!("Rust ghb/{}", env!("CARGO_PKG_VERSION")),
    );
    modified_response
}