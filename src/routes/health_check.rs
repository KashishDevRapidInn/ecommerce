use actix_web::HttpResponse;

/******************************************/
// Health check route
/******************************************/
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
