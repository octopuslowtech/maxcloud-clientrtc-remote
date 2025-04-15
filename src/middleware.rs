use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
};
use std::{
    future::{ready, Ready, Future},
    pin::Pin,
    sync::Arc,
};
use tokio::sync::Mutex;
use crate::models::AppState;

// Middleware xác thực
pub struct AuthMiddleware<S> {
    service: S,
}

// Factory để tạo AuthMiddleware
pub struct AuthMiddlewareFactory {
    pub exclude_routes: Vec<String>,
    pub app_state: Arc<Mutex<AppState>>,
}

// Cài đặt Transform trait cho AuthMiddlewareFactory
impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        Box::pin(async move {
            // Lấy kết quả từ future
            fut.await
        })
    }
}

// Đổi tên từ JwtAuthMiddleware thành AuthMiddleware
pub struct AuthenticationMiddleware {
    pub exclude_routes: Vec<String>,
    pub app_state: Arc<Mutex<AppState>>,
}

impl<S, B> Transform<S, ServiceRequest> for AuthenticationMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = AuthenticationMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddlewareService {
            service,
            exclude_routes: self.exclude_routes.clone(),
            app_state: self.app_state.clone(),
        }))
    }
}

pub struct AuthenticationMiddlewareService<S> {
    service: S,
    exclude_routes: Vec<String>,
    app_state: Arc<Mutex<AppState>>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Kiểm tra nếu route nằm trong danh sách loại trừ
        let path = req.path().to_string();
        let exclude = self.exclude_routes.iter().any(|route| path.starts_with(route));

        if exclude {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        // Clone app_state để sử dụng trong future
        let app_state = self.app_state.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            // Chỉ kiểm tra xem đã đăng nhập chưa (có phiên làm việc chưa)
            // không cần xác thực JWT
            let state = app_state.lock().await;
            if state.hub_connection.is_none() {
                return Err(ErrorUnauthorized("Bạn cần đăng nhập để truy cập"));
            }
            
            // Tiếp tục xử lý request
            drop(state);
            fut.await
        })
    }
} 