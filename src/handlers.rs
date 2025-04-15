use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::models::{AppState, LoginQuery, LoginResponse, ApiResponseV2, DeviceStatus, ConnectDeviceRequest};
use crate::connect_to_signalr;

pub async fn hello() -> impl Responder {
    let response = ApiResponseV2 {
        status_code: 200,
        message: "Hello World".to_string(),
        success: true,
        data: None::<()>,
    };
    
    HttpResponse::Ok().json(response)
}

pub async fn login(
    query: web::Query<LoginQuery>,
    app_state: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    // Kiểm tra xem đã login chưa
    let state = app_state.lock().await;
    if state.jwt_token.is_some() && state.hub_connection.is_some() {
        let response = ApiResponseV2 {
            status_code: 200,
            message: "Đã đăng nhập".to_string(),
            success: true,
            data: None::<()>,
        };
        return HttpResponse::Ok().json(response);
    }
    drop(state);

    let key = &query.key;
    
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .tls_built_in_root_certs(false)
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .build()
        .unwrap();
    
    let login_result = client
        .get(format!("{}/Octopus/login/{}", "https://api.maxcloudphone.com", key))
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
                            Ok(mut hub_connection) => {
                                let _message_handler = hub_connection.register("MESSAGE".to_string(), |ctx| {
                                    if let Ok(message) = ctx.argument::<String>(0) {
                                        println!("Nhận được tin nhắn: {}", message);
                                    } else {
                                        println!("Không thể đọc tin nhắn");
                                    }
                                });
                                
                                state.hub_connection = Some(hub_connection);
                                
                                let response = ApiResponseV2 {
                                    status_code: 200,
                                    message: "Đăng nhập thành công và đã kết nối đến SignalR".to_string(),
                                    success: true,
                                    data: None::<()>,
                                };
                                
                                HttpResponse::Ok().json(response)
                            }
                            Err(e) => {
                                let response = ApiResponseV2 {
                                    status_code: 500,
                                    message: format!("Đăng nhập thành công nhưng không thể kết nối đến SignalR: {}", e),
                                    success: false,
                                    data: None::<()>,
                                };
                                
                                HttpResponse::InternalServerError().json(response)
                            }
                        }
                    } else {
                        let response = ApiResponseV2 {
                            status_code: 401,
                            message: login_data.messages.first().unwrap_or(&"Đăng nhập không thành công".to_string()).to_string(),
                            success: false,
                            data: None::<()>,
                        };
                        
                        HttpResponse::Unauthorized().json(response)
                    }
                }
                Err(e) => {
                    println!("Parse error: {}", e);
                    
                    let response = ApiResponseV2 {
                        status_code: 400,
                        message: format!("Lỗi khi xử lý phản hồi: {}", e),
                        success: false,
                        data: None::<()>,
                    };
                    
                    HttpResponse::BadRequest().json(response)
                }
            }
        }
        Err(e) => {
            let response = ApiResponseV2 {
                status_code: 500,
                message: format!("Lỗi kết nối đến máy chủ xác thực: {}", e),
                success: false,
                data: None::<()>,
            };
            
            HttpResponse::InternalServerError().json(response)
        }
    }
}

// Endpoint để lấy danh sách thiết bị
pub async fn get_devices(
    app_state: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    // Lấy danh sách thiết bị từ state
    let state = app_state.lock().await;
    let devices = state.devices.clone();
    
    // Trả về response
    let response = ApiResponseV2 {
        status_code: 200,
        message: "Lấy danh sách thiết bị thành công".to_string(),
        success: true,
        data: Some(devices),
    };
    
    HttpResponse::Ok().json(response)
}

// Endpoint để kết nối đến thiết bị
pub async fn connect_device(
    req: web::Json<ConnectDeviceRequest>,
    app_state: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    let device_id = &req.device_id;
    
    let mut state = app_state.lock().await;
    
    // Kiểm tra xem thiết bị có tồn tại không
    let device_index = state.devices.iter().position(|d| d.device_id == *device_id);
    
    if let Some(index) = device_index {
        // Kiểm tra xem thiết bị đã kết nối chưa
        let device = &state.devices[index];
        
        if device.status == DeviceStatus::Connected {
            let response = ApiResponseV2::<()> {
                status_code: 400,
                message: "Thiết bị đã được kết nối".to_string(),
                success: false,
                data: None,
            };
            
            return HttpResponse::BadRequest().json(response);
        }
        
        // Cập nhật trạng thái thiết bị (TODO: thêm logic kết nối thực tế)
        state.devices[index].status = DeviceStatus::Connected;
        
        // Trả về response thành công
        let response = ApiResponseV2 {
            status_code: 200,
            message: "Kết nối đến thiết bị thành công".to_string(),
            success: true,
            data: Some(state.devices[index].clone()),
        };
        
        HttpResponse::Ok().json(response)
    } else {
        // Thiết bị không tồn tại
        let response = ApiResponseV2::<()> {
            status_code: 404,
            message: "Không tìm thấy thiết bị".to_string(),
            success: false,
            data: None,
        };
        
        HttpResponse::NotFound().json(response)
    }
} 