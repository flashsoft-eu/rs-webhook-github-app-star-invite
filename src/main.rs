use dotenv::dotenv;
mod ghb;

use ghb::server::server_run; 
use ghb::github::check_auth;


fn main() {
    dotenv().ok();

    if !check_auth() {
        println!("Github auth failed! Check ENV vars!, exiting...");
        return;
    }

    server_run();
}
