use serde::{Deserialize, Serialize};
use signalr_client::SignalRClient;

// Định nghĩa kiểu tạm thời cho PeerConnection (sẽ thay thế sau)
pub type PeerConnection = Option<()>;

// Định nghĩa trạng thái thiết bị
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Connected,
    Disconnected,
    Offline,
}

// Định nghĩa struct thiết bị
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub device_id: String,
    pub status: DeviceStatus,
}

// Định nghĩa struct AppState để lưu trạng thái ứng dụng
pub struct AppState {
    pub peer_connection: PeerConnection,
    pub hub_connection: Option<SignalRClient>,
    pub jwt_token: Option<String>,
    pub devices: Vec<Device>,
}

// Định nghĩa struct cho phản hồi đăng nhập
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub data: LoginData,
    pub messages: Vec<String>,
    pub succeeded: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginData {
    pub token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct LoginQuery {
    pub key: String,
}

// Struct response tiêu chuẩn
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

// Struct response mới theo yêu cầu
#[derive(Serialize)]
pub struct ApiResponseV2<T> {
    pub status_code: i32,
    pub message: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

// Struct cho connect device request
#[derive(Deserialize)]
pub struct ConnectDeviceRequest {
    pub device_id: String,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            peer_connection: None,
            hub_connection: None,
            jwt_token: None,
            devices: vec![
                Device {
                    device_id: "device001".to_string(),
                    status: DeviceStatus::Offline,
                },
                Device {
                    device_id: "device002".to_string(),
                    status: DeviceStatus::Offline,
                },
                Device {
                    device_id: "device003".to_string(), 
                    status: DeviceStatus::Offline,
                },
            ],
        }
    }
} 