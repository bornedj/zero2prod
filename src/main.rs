use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // panic if we cannot get a config file for the database
    let configuration = get_configuration().expect("Failed to read configuration file");

    // setting up database connection
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    // address coming from config file
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    // building the email client
    let timeout = configuration.email_client.timeout();
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );
    let listener = TcpListener::bind(address).expect("Failed to bind to random port.");
    run(listener, connection_pool, email_client)?.await
}
