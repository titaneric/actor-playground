use actix_web::{post, web, HttpResponse};
use log::info;
use serde::Deserialize;

#[derive(Deserialize)]
struct CompileReq {
    code: String,
}
#[post("/compile")]
async fn echo(compile_req: web::Json<CompileReq>) -> HttpResponse {
    info!("Welcome {}!", compile_req.code);
    HttpResponse::Ok().json("ok")
}
