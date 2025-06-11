use shuttle_runtime::{SecretStore, Error as ShuttleError};
use anyhow::anyhow;

mod ghb;
use ghb::server::server_run;
use ghb::github::check_auth;
use ghb::config::init_config;


struct RouilleServiceWrapper;

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for RouilleServiceWrapper {
    async fn bind(
        mut self,
        addr: std::net::SocketAddr, // `addr` here is the address Shuttle wants your service to listen on
    ) -> Result<(), shuttle_runtime::Error> {
        println!("RouilleServiceWrapper received bind call for: {}", addr);
        tokio::task::spawn_blocking(move || {
            server_run(addr, tokio::runtime::Handle::current());
        })
        .await
        .map_err(|e| ShuttleError::from(anyhow!("Rouille server blocking task failed: {}", e)))?;
        Ok(())
    }
}


#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
) -> Result<RouilleServiceWrapper, ShuttleError> {

    init_config(&secret_store);

    if !check_auth() {
        return Err(ShuttleError::from(anyhow!("GitHub authentication failed! Check ENV vars!")));
    }

    tokio::spawn(async move {
        ghb::tokio_worker::periodic_refresh_inst_token().await;
    });

    println!("Shuttle main function finished setup.");

    Ok(RouilleServiceWrapper)
}