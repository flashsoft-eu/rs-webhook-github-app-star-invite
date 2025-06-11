use crate::ghb::github::handle_hook;
use rouille::Response;
use std::net::SocketAddr;
use tokio::runtime::Handle as TokioHandle;


/// Starts the Rouille server, listening on the provided address.
/// This function is blocking.
pub fn server_run(addr: SocketAddr, runtime_handle: TokioHandle) {
    println!("Starting Rouille server on {}", addr);
    rouille::start_server_with_pool(addr, Some(3), move |request| {
        if request.url() == "/" {
            Response::text("Github Hook Bot is running!")
        } else if request.url() == "/github-webhook" {
            // Ensure handle_hook returns a rouille::Response
            // Handle any potential errors from handle_hook gracefully
            handle_hook(&request, runtime_handle.clone())
        } else {
            Response::empty_404()
        }
    });
}
