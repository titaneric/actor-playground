mod compile;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use compile::runner::runner_client::RunnerClient;
use compile::runner::{ExecuteRequest, ExecuteResponse};
use log::info;
use std::sync::Arc;
use std::{env, fs::read_to_string, io::Read};
use std::{io::Write, sync::Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let wc = Arc::new(Mutex::new(compile::WorkerClient::new().await));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(wc.clone()))
            .wrap(middleware::Logger::default())
            .service(hello)
            .service(compile::compile_handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
