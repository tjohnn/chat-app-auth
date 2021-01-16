use dotenv::dotenv;
use actix_web::{HttpServer, App, middleware};
use chat_app::{routes, AppState, ServiceContainer};
use mongodb::Client;


#[actix_rt::main]
async fn main() -> std::io::Result<()>  {
    dotenv().ok();
    env_logger::init();

    // setup database
    let db_url = std::env::var("MONGO_URI").expect("Unable to get MONGO_URI for db connection from .env");
    let client = Client::with_uri_str(db_url.as_str()).await;
    let client = client.expect(format!("Unable to donnect to database: {}", db_url).as_str());
    let db_name = std::env::var("DB_NAME").expect("Unable to get DB_NAME for db connection from .env");
    let database = client.database(db_name.as_str());
    let state = AppState {service_container: ServiceContainer::new(database)};

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(state.clone())
            .configure(routes::api::config)
    })
        .bind("127.0.0.1:8088")
        .expect("Unable to bind to port 8088")
        .run()
        .await
}
