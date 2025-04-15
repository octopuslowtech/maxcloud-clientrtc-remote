use crate::client::{Authentication, ConnectionConfiguration};
use crate::execution::UpdatableActionStorage; 
use crate::protocol::negotiate::NegotiateResponseV0;
use base64::{engine::general_purpose, Engine};
use serde::{de::DeserializeOwned, Serialize};

const WEB_SOCKET_TRANSPORT: &str = "WebSockets";
const TEXT_TRANSPORT_FORMAT: &str = "Text";

#[derive(Clone, Debug)]
pub struct ConnectionData {
    endpoint: String,
    connection_id: String,
}

impl ConnectionData {
    pub fn get_endpoint(&self) -> String {
        self.endpoint.clone()
    }

    #[allow(dead_code)]
    pub fn get_connection_id(&self) -> String {
        self.connection_id.clone()
    }
}

pub trait Communication : Clone {
    async fn connect(configuration: &ConnectionData) -> Result<Self, String>;
    async fn send<T: Serialize>(&mut self, data: T) -> Result<(), String>;
    fn get_storage(&self) -> Result<UpdatableActionStorage, String>;
    fn disconnect(&mut self);
}

pub struct HttpClient {
    
}

impl HttpClient {
    pub(crate) async fn negotiate(options: ConnectionConfiguration) -> Result<ConnectionData, String> {
        // Bỏ qua negotiate, tạo kết nối WebSocket trực tiếp
        Ok(ConnectionData {
            endpoint: options.get_socket_url(),
            connection_id: String::new(), // Connection ID không cần thiết khi không negotiate
        })
    }

    fn create_configuration(endpoint: String, _negotiate: NegotiateResponseV0) -> Option<ConnectionData> {
        // Luôn trả về Some vì chúng ta đã biết server hỗ trợ WebSocket
        Some(ConnectionData {
            endpoint: endpoint,
            connection_id: String::new(),
        })
    }

    pub async fn post<T: 'static + DeserializeOwned + Send>(endpoint: String, _authentication: Authentication) -> Result<T, String> {
        // Phương thức này sẽ không được sử dụng nữa khi bỏ qua negotiate
        Err("Direct WebSocket connection, POST not needed".to_string())
    }
}