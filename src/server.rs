mod compile;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use runner::runner_client::{RunnerClient};
use runner::{ExecuteRequest, ExecuteResponse};
use tonic::{transport::Server, Request as TonicRequest, Response as TonicResponse, Status};

pub mod runner {
    tonic::include_proto!("runner");
}

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     std::env::set_var("RUST_LOG", "info");
//     env_logger::init();
//     let wc = web::Data::new(compile::WorkerClient::new());

//     HttpServer::new(move || {
//         App::new()
//             .app_data(wc.clone())
//             .wrap(middleware::Logger::default())
//             .service(hello)
//             .service(compile::compile_handler)
//     })
//     .bind(("127.0.0.1", 8080))?
//     .run()
//     .await
// }
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RunnerClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(ExecuteRequest {
        binary: "Tonic".into(),
    });

    let response = client.execute(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
