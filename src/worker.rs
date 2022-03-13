mod run;
use actix_web::{get, middleware, App, HttpResponse, HttpServer, Responder};
use run::runner::runner_server::RunnerServer;
use run::RunnerImpl;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let addr = "[::1]:50051".parse()?;
    let greeter = RunnerImpl::default();
    let service = RunnerServer::new(greeter);

    Server::builder()
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}
// #[get("/")]
// async fn hello() -> impl Responder {
//     HttpResponse::Ok().body("Hello world!")
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     std::env::set_var("RUST_LOG", "info");
//     env_logger::init();
//     HttpServer::new(|| {
//         App::new()
//             .wrap(middleware::Logger::default())
//             .service(run::run_handler)
//             .service(hello)
//     })
//     .bind(("127.0.0.1", 7070))?
//     .run()
//     .await
// }
