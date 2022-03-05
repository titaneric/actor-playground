use actix_web::{post, web, HttpResponse};
use log::info;
use serde::Deserialize;

#[derive(Deserialize)]
struct RunReq {
    built_binary: Vec<u8>,
}
#[post("/run")]
async fn run_handler(run_req: web::Json<RunReq>) -> HttpResponse {
    info!("Welcome {:?}!", run_req.built_binary);
    HttpResponse::Ok().json("ok")
}
