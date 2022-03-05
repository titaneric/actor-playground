use actix_web::{post, web, HttpResponse};
use log::info;
use serde::Deserialize;
use awc::Client;

const WORKER_URL: &str = "http://localhost:7070/run";
#[derive(Deserialize)]
struct CompileReq {
    code: String,
}
#[post("/compile")]
async fn compile_handler(compile_req: web::Json<CompileReq>) -> HttpResponse {
    info!("Welcome {}!", compile_req.code);
    let request = serde_json::json!({
        "built_binary":compile_req.code.as_bytes(),
    });
    
    let mut client = awc::Client::default();
    let response = client.post(WORKER_URL)
        .send_json(&request)
        .await.unwrap();

    HttpResponse::Ok().json("ok")
}
