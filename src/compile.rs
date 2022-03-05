use actix_web::{post, web, HttpRequest, HttpResponse};
use log::info;
use serde::Deserialize;
use std::sync::Mutex;
// use awc::Client;
use reqwest::Client;
pub struct WorkerClient {
    pub client: Client,
}

impl WorkerClient {
    pub fn new() -> WorkerClient {
        WorkerClient {
            client: Client::new(),
        }
    }
}

const WORKER_URL: &str = "http://localhost:7070/run";
#[derive(Deserialize)]
struct CompileReq {
    code: String,
}
#[post("/compile")]
async fn compile_handler(
    compile_req: web::Json<CompileReq>,
    worker_client: HttpRequest,
) -> HttpResponse {
    let request = serde_json::json!({
        "built_binary":compile_req.code.as_bytes(),
    });
    let client = &worker_client
        .app_data::<web::Data<WorkerClient>>()
        .unwrap()
        .client;

    client.post(WORKER_URL).json(&request).send().await.unwrap();

    info!("Welcome {}!", compile_req.code);
    HttpResponse::Ok().json("ok")
}
