use actix_web::{post, web, HttpResponse};
use futures::StreamExt;
use log::info;
use serde::Deserialize;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{set_permissions, File},
    io::Write,
    path::{Path, PathBuf},
};
use tempdir::TempDir;
use tokio::process::Command;

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
    let run_req: RunReq = serde_json::from_slice(&bytes).unwrap();

    info!("Chunk: {:?}", run_req.built_binary.len());
    let filename = "runner";
    let exe_name = write_byte_stream(&run_req.built_binary, filename);
    let mut command = Command::new(exe_name);
    let output = command.output().await.unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    info!("{:?}", stdout);
    // info!("{:?}", std::str::from_utf8(&output.stderr));
    // info!("{:?}", output.status);

    HttpResponse::Ok().json(stdout)
}
fn write_byte_stream(buf: &[u8], filename: &str) -> PathBuf {
    let tmp_dir = TempDir::new("rust-run").unwrap();
    let filename = tmp_dir.into_path().join(filename);
    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o755)
        .open(&filename)
        .unwrap();
    f.write_all(buf).unwrap();
    info!("{:#o}", f.metadata().unwrap().permissions().mode());
    filename
}
