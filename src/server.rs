mod compile;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let wc = web::Data::new(compile::WorkerClient::new());

    HttpServer::new(move || {
        App::new()
            .app_data(wc.clone())
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
