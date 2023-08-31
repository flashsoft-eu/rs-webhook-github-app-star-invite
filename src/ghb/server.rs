

use rouille::Response;

use crate::ghb::github::handle_hook;

pub fn server_run() {
    rouille::start_server_with_pool("0.0.0.0:4440", Some(3),  move |request| {
       if request.url() == "/" {
           Response::text("Github Hook Bot is running!")
       } else if request.url() == "/github-webhook" {
        handle_hook(&request)
       } else {
           Response::empty_404()
       }
    });
}