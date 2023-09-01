use tokio::time::Duration;


use crate::ghb::github::get_installation_token;

pub async fn periodic_refresh_inst_token() {
    let interval = Duration::from_secs(60 * 3);

    let mut interval_count = 0;
    loop {
        interval_count += 1;

        get_installation_token();

        println!("Exec refresh token interval count: {}", interval_count);
        tokio::time::sleep(interval).await;
    }
}
