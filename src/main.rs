use dotenv::dotenv;
mod ghb;

use ghb::server::server_run; 
use ghb::github::check_auth;
use ghb::tokio_worker::periodic_refresh_inst_token;

#[tokio::main]
async fn main() {
    dotenv().ok();

    if !check_auth() {
        println!("Github auth failed! Check ENV vars!, exiting...");
        return;
    }

    tokio::spawn(async move {
        periodic_refresh_inst_token().await;
    });

     
    server_run();
}
