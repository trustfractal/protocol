use native_tls::*;
use postgres::Client;

pub fn connect(url: &str) -> anyhow::Result<Client> {
    let connector = TlsConnector::builder()
        // Heroku generates a self-signed certificate for the machines running this.
        // We need to allow that as an "invalid" certificate.
        .danger_accept_invalid_certs(true)
        .build()?;
    let connector = postgres_native_tls::MakeTlsConnector::new(connector);

    Ok(url
        .parse::<postgres::Config>()?
        .ssl_mode(postgres::config::SslMode::Require)
        .connect(connector)?)
}
