use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use signalr_client::SignalRClient;

// URL backend
const BACKEND_URL: &str = "http://localhost:7051";

// Trạng thái WebRTC (sẽ dùng sau khi mở rộng)
// Định nghĩa kiểu tạm thời cho PeerConnection (sẽ thay thế sau)
type PeerConnection = Option<()>;

// Định nghĩa struct AppState để lưu trạng thái ứng dụng
struct AppState {
    peer_connection: PeerConnection,
    hub_connection: Option<SignalRClient>,
    jwt_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginResponse {
    data: LoginData,
    messages: Vec<String>,
    succeeded: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginData {
    token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("hello world")
}

#[get("/login")]
async fn login(
    query: web::Query<LoginQuery>,
    app_state: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    let key = &query.key;
    
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) 
        .build()
        .unwrap();
    
    let login_result = client
        .get(format!("{}/Octopus/login/{}", BACKEND_URL, key))
        .header("Content-Type", "application/json")
        .send()
        .await;
    
    match login_result {
        Ok(response) => {
            let text = response.text().await.unwrap();
            println!("Response text: {}", text);
            
            match serde_json::from_str::<LoginResponse>(&text) {
                Ok(login_data) => {
                    if login_data.succeeded {
                        let token = login_data.data.token;
                        
                        // Lưu JWT token vào AppState
                        let mut state = app_state.lock().await;
                        state.jwt_token = Some(token.clone());
                        
                        // Kết nối đến SignalR
                        match connect_to_signalr(&token).await {
                            Ok(hub_connection) => {
                                state.hub_connection = Some(hub_connection);
                                HttpResponse::Ok().json(serde_json::json!({
                                    "success": true,
                                    "message": "Đăng nhập thành công và đã kết nối đến SignalR",
                                }))
                            }
                            Err(e) => {
                                HttpResponse::InternalServerError().json(serde_json::json!({
                                    "success": false,
                                    "message": format!("Đăng nhập thành công nhưng không thể kết nối đến SignalR: {}", e),
                                }))
                            }
                        }
                    } else {
                        HttpResponse::Unauthorized().json(serde_json::json!({
                            "success": false,
                            "message": login_data.messages.first().unwrap_or(&"Đăng nhập không thành công".to_string()).to_string(),
                        }))
                    }
                }
                Err(e) => {
                    println!("Parse error: {}", e);
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "success": false,
                        "message": format!("Lỗi khi xử lý phản hồi: {}", e),
                    }))
                }
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Lỗi kết nối đến máy chủ xác thực: {}", e),
            }))
        }
    }
}

#[derive(Deserialize)]
struct LoginQuery {
    key: String,
}

async fn connect_to_signalr(token: &str) -> Result<SignalRClient, Box<dyn std::error::Error>> {
    println!("Bắt đầu kết nối đến SignalR...  {}", token);

    let url = BACKEND_URL.trim_start_matches("http://");
    let parts: Vec<&str> = url.split(':').collect();
    let domain = parts[0];
    let port = parts[1].parse::<i32>().unwrap();

    println!("Domain: {}", domain);
    println!("Port: {}", port);
    println!("Token: {}", token);

    let client = SignalRClient::connect_with(domain, "deviceRHub", |c| {
        c.with_port(port);
        c.unsecure();
        c.with_query_param("type".to_string(), "client".to_string());
        c.with_access_token(token.to_string());
    }).await?;

    println!("Kết nối đến SignalR thành công");

    Ok(client)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    println!("Khởi động server tại http://localhost:8081");

    let state = Arc::new(Mutex::new(AppState {
        peer_connection: None,
        hub_connection: None,
        jwt_token: None,
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(hello)
            .service(login)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
