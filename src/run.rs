use actix_web::{post, web, HttpResponse};
use log::info;
use serde::Deserialize;
use futures::StreamExt;

#[derive(Deserialize)]
struct RunReq {
    built_binary: Vec<u8>,
}
#[post("/run")]
async fn run_handler(mut body: web::Payload) -> HttpResponse {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        let item = item.unwrap();
        bytes.extend_from_slice(&item);
    }
    let run_req: RunReq= serde_json::from_slice(&bytes).unwrap();
    println!("Chunk: {:?}",run_req.built_binary.len() );

    HttpResponse::Ok().json("ok")
}
