mod models;
mod middleware;
mod handlers;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use tokio::sync::Mutex;
use signalr_client::SignalRClient;
use models::AppState;
use middleware::AuthenticationMiddleware;

// URL backend
const BACKEND_URL: &str = "https://api.maxcloudphone.com";

async fn connect_to_signalr(token: &str) -> Result<SignalRClient, Box<dyn std::error::Error>> {
    println!("Connecting to SignalR hub...");
    println!("URL: {}", BACKEND_URL);
    
    let client = SignalRClient::connect_with("api.maxcloudphone.com", "deviceRHub", |c| {
        c.with_port(443);
        c.secure();  // Sử dụng HTTPS/WSS
        c.with_query_param("type".to_string(), "client".to_string());
        c.with_access_token(token.to_string());
    }).await?;

    println!("Connected to SignalR successfully!");
    Ok(client)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Khởi động server tại http://localhost:1510");

    let state = Arc::new(Mutex::new(AppState::new()));

    let excluded_routes = vec!["/hello".to_string(), "/login".to_string()];

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(AuthenticationMiddleware {
                exclude_routes: excluded_routes.clone(),
                app_state: state.clone(),
            })
            .service(web::resource("/hello").route(web::get().to(handlers::hello)))
            .service(web::resource("/login").route(web::get().to(handlers::login)))
            .service(web::resource("/get-devices").route(web::get().to(handlers::get_devices)))
            .service(web::resource("/connect-device").route(web::post().to(handlers::connect_device)))
    })
    .bind("127.0.0.1:1510")?
    .run()
    .await
}
