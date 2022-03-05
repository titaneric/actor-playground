use actix_web::{post, web, HttpRequest, HttpResponse};
use cargo::{
    core::{compiler::CompileMode, Workspace},
    ops::{compile, CompileOptions},
    util::Config,
};
use log::info;
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use std::{path::PathBuf, sync::Mutex};
use std::fs::{File, metadata};
use std::io::Read;
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
    let config = Config::default().unwrap();
    let manifest_path = Path::new("/home/titaneric/sources/actor-playground/compile_target/Cargo.toml");
    let workspace = Workspace::new(manifest_path.into(), &config).unwrap();
    let compile_option = CompileOptions::new(&config, CompileMode::Build).unwrap();
    let compile_result = compile(&workspace, &compile_option).unwrap();
    let built_binary_paths = compile_result
        .binaries
        .iter()
        .map(|unit_output| unit_output.path.to_str().unwrap())
        .collect::<Vec<&str>>();
    info!("{:?}!", built_binary_paths);

    let request = serde_json::json!({
        "built_binary":get_file_as_byte_vec(&String::from(built_binary_paths[0])),
    });
    let client = &worker_client
        .app_data::<web::Data<WorkerClient>>()
        .unwrap()
        .client;

    client.post(WORKER_URL).json(&request).send().await.unwrap();

    info!("Welcome {}!", compile_req.code);
    HttpResponse::Ok().json("ok")
}
fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}
