use native_tls::*;
use postgres::Client;

pub fn connect(url: &str) -> anyhow::Result<Client> {
    let connector = TlsConnector::builder()
        // Heroku generates a self-signed certificate for the machines running this.
        // We need to allow that as an "invalid" certificate.
        .danger_accept_invalid_certs(true)
        .build()?;
    let connector = postgres_native_tls::MakeTlsConnector::new(connector);

    let mut connection = url
        .parse::<postgres::Config>()?
        .ssl_mode(postgres::config::SslMode::Require)
        .connect(connector)?;
    connection.execute("SET statement_timeout=10000", &[])?;

    Ok(connection)
}
