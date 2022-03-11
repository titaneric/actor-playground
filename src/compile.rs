use actix_web::{error, post, web, HttpRequest, Responder, Result as ActixWebResult};
use cargo::{
    core::{compiler::CompileMode, Workspace},
    ops::{self, CompileOptions, NewOptions},
    util::Config,
};
use futures::TryFutureExt;
use log::info;
use proc_macro2::Span;
use quote::quote;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{metadata, File},
    path::Path,
};
// use anyhow::Result;
use runner::runner_client::RunnerClient;
use runner::{ExecuteRequest, ExecuteResponse};
use std::{env, fs::read_to_string, io::Read};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
use syn::fold::Fold;
use syn::{token::Pub, VisPublic, Visibility};
use syn::{File as SynFile, ItemFn};
use tempdir::TempDir;
use thiserror::Error;
use tonic::transport;
pub mod runner {
    tonic::include_proto!("runner");
}
pub struct WorkerClient {
    pub client: RunnerClient<transport::Channel>,
}

impl WorkerClient {
    pub async fn new() -> WorkerClient {
        WorkerClient {
            client: RunnerClient::new(
                transport::Channel::from_static("http://[::1]:50051")
                    .connect()
                    .await
                    .unwrap(),
            ),
        }
    }
}

const WORKER_URL: &str = "http://localhost:7070/run";
#[derive(Deserialize)]
struct CompileReq {
    code: String,
}
#[derive(Deserialize, Serialize)]
struct CompileResponse {
    stdout: String,
}
#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Failed to build the given source code")]
    BuildBinaryError {},
    #[error("Unable to retrieve worker client app data")]
    RetrieveAppDataFailure {},
    #[error("Failed to lock the worker client mutex")]
    LockMutexError {},
}
impl error::ResponseError for CompileError {}

#[post("/compile")]
async fn compile_handler(
    compile_req: web::Json<CompileReq>,
    worker_client: web::Data<Arc<Mutex<WorkerClient>>>,
) -> Result<impl Responder, CompileError> {
    info!("{:?}", compile_req.code.as_str());
    let built_binary = create_compile_sandbox(compile_req.code.clone()).await;
    let mut client = worker_client.lock().unwrap().client.clone();
    let request = tonic::Request::new(ExecuteRequest {
        binary: built_binary,
    });

    let response = client.execute(request).await.unwrap();

    info!("{:?}", response);
    Ok("ok")
}
fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    buffer
}
fn write_src_code(src_code: String, filename: &Path) {
    let mut f = File::create(&filename).expect("no file found");
    f.write_all(&src_code.into_bytes()).unwrap();
}
async fn create_compile_sandbox(src_code: String) -> Vec<u8> {
    // create tmp cargo package
    let tmp_dir = TempDir::new("rust-build").unwrap();

    env::set_var("CARGO_INCREMENTAL", "false");
    env::set_var("RUSTC_WRAPPER", "/home/titaneric/.cargo/bin/sccache");

    const PLAYGROUND_PACKAGE_NAME: &str = "playground";
    const PLAYGROUND_MAIN_FN: &str = r#"
        fn main() {
            playground::main();
        }
    "#;

    let config = Config::default().unwrap();
    let cargo_tmp = tmp_dir.into_path().join(PLAYGROUND_PACKAGE_NAME);
    let new_option =
        NewOptions::new(None, false, true, cargo_tmp.clone(), None, None, None).unwrap();
    ops::new(&new_option, &config).unwrap();
    // let modified_main_fn = parse_src(cargo_tmp.as_path().join("src").join("main.rs").as_path());
    let modified_main_fn = parse_src(src_code);
    write_src_code(
        modified_main_fn,
        cargo_tmp.as_path().join("src").join("lib.rs").as_path(),
    );
    write_src_code(
        String::from(PLAYGROUND_MAIN_FN),
        cargo_tmp.as_path().join("src").join("main.rs").as_path(),
    );

    // build it
    let manifest_path = cargo_tmp.join("Cargo.toml");
    let workspace = Workspace::new(manifest_path.as_path(), &config).unwrap();
    let compile_option = CompileOptions::new(&config, CompileMode::Build).unwrap();
    let compile_result = ops::compile(&workspace, &compile_option).unwrap();
    let built_binary_paths = compile_result
        .binaries
        .iter()
        .map(|unit_output| unit_output.path.to_str().unwrap())
        .collect::<Vec<&str>>();

    get_file_as_byte_vec(&String::from(built_binary_paths[0]))
}

struct PublicizeMainFn;

impl Fold for PublicizeMainFn {
    fn fold_file(&mut self, file: SynFile) -> SynFile {
        SynFile {
            items: file
                .items
                .iter()
                .map(|item| match item {
                    syn::Item::Fn(item_fn) => self.fold_item_fn(item_fn.to_owned()).into(),
                    _ => item.to_owned(),
                })
                .collect(),
            ..file
        }
    }
    fn fold_item_fn(&mut self, item_fn: ItemFn) -> ItemFn {
        match item_fn.sig.ident.to_string().as_str() {
            "main" => ItemFn {
                vis: Visibility::Public(VisPublic {
                    pub_token: Pub {
                        span: Span::call_site(),
                    },
                }),
                ..item_fn
            },
            _ => item_fn,
        }
    }
}
fn parse_src(src_code: String) -> String {
    // let src_code = read_to_string(src_code_path).unwrap();
    let ast: SynFile = syn::parse_str(&src_code).unwrap();
    // info!("{:#?}", ast);
    let modified_main_fn = PublicizeMainFn.fold_file(ast);
    info!("{}", quote! {#modified_main_fn});
    let modified_main_fn: String = quote! {#modified_main_fn}.to_string();
    modified_main_fn
}
