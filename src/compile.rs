use actix_web::{post, web, HttpRequest, HttpResponse};
use cargo::{
    core::{compiler::CompileMode, Workspace},
    ops::{self, CompileOptions, NewOptions},
    util::Config,
};
use log::{info, debug};
use reqwest::Client;
use serde::Deserialize;
use std::{io::Read, fs::read_to_string};
use std::sync::Mutex;
use std::{
    collections::HashMap,
    fs::{metadata, File},
    path::Path,
};
use syn::visit::{self, Visit};
use syn::{File as SynFile, ItemFn};
use tempdir::TempDir;
pub struct WorkerClient {
    pub client: Mutex<Client>,
}

impl WorkerClient {
    pub fn new() -> WorkerClient {
        WorkerClient {
            client: Mutex::new(Client::new()),
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
    let built_binary = create_compile_sandbox().await;

    let request = serde_json::json!({
        "built_binary":built_binary,
    });
    let client = &worker_client
        .app_data::<web::Data<WorkerClient>>()
        .unwrap()
        .client;

    client
        .lock()
        .unwrap()
        .post(WORKER_URL)
        .json(&request)
        .send()
        .await
        .unwrap();

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
async fn create_compile_sandbox() -> Vec<u8> {
    let mut config = Config::default().unwrap();
    let build_config = {
        let mut map = HashMap::new();
        map.insert(
            String::from("RUSTC_WRAPPER"),
            String::from("/home/titaneric/.cargo/bin/sccache"),
        );
        map
    };
    config.set_env(build_config);

    // create tmp cargo package
    let tmp_dir = TempDir::new("rust-build").unwrap();
    // let cargo_tmp = tmp_dir.into_path().join("cargo");
    let cargo_tmp = tmp_dir.into_path().join("playground");
    let new_option =
        NewOptions::new(None, true, false, cargo_tmp.clone(), None, None, None).unwrap();
    ops::new(&new_option, &config).unwrap();
    parse_src(cargo_tmp.as_path().join("src").join("main.rs").as_path());

    // build it
    let manifest_path = cargo_tmp.join("Cargo.toml");
    let workspace = Workspace::new(&manifest_path.as_path(), &config).unwrap();
    let compile_option = CompileOptions::new(&config, CompileMode::Build).unwrap();
    let compile_result = ops::compile(&workspace, &compile_option).unwrap();
    let built_binary_paths = compile_result
        .binaries
        .iter()
        .map(|unit_output| unit_output.path.to_str().unwrap())
        .collect::<Vec<&str>>();

    get_file_as_byte_vec(&String::from(built_binary_paths[0]))
}


struct FnVisitor;

impl<'ast> Visit<'ast> for FnVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        println!("Function with name={}", node.sig.ident);

        // Delegate to the default impl to visit any nested functions.
        visit::visit_item_fn(self, node);
    }
}

fn parse_src(src_code_path: &Path) {
    let src_code = read_to_string(src_code_path).unwrap();
    let ast: syn::File = syn::parse_str(&src_code).unwrap();
    info!("{:#?}", ast);
    FnVisitor.visit_file(&ast);

}
