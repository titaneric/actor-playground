use actix_web::{post, web, HttpResponse, Responder, Result};
use futures::StreamExt;
use log::info;
use runner::runner_server::{Runner, RunnerServer};
use runner::{ExecuteRequest, ExecuteResponse};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
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
use tonic::{transport::Server, Request as TonicRequest, Response as TonicResponse, Status};

pub mod runner {
    tonic::include_proto!("runner");
}

#[derive(Debug, Default)]
pub struct RunnerImpl {}

#[tonic::async_trait]
impl Runner for RunnerImpl {
    async fn execute(
        &self,
        request: TonicRequest<ExecuteRequest>, // Accept request of type HelloRequest
    ) -> Result<TonicResponse<ExecuteResponse>, Status> {
        // Return an instance of type HelloReply
        println!("Got a binary size is: {:?}", request.get_ref().binary.len());

        let reply = run_handler(&request.get_ref().binary).await.unwrap();
        Ok(TonicResponse::new(reply)) // Send back our formatted greeting
    }
}

async fn run_handler(built_binary: &[u8]) -> Result<ExecuteResponse> {
    // let mut bytes = web::BytesMut::new();
    // while let Some(item) = body.next().await {
    //     let item = item.unwrap();
    //     bytes.extend_from_slice(&item);
    // }
    // let run_req: RunReq = serde_json::from_slice(&bytes).unwrap();

    // info!("Chunk: {:?}", run_req.built_binary.len());
    let filename = "runner";
    let exe_name = write_byte_stream(built_binary, filename);
    let mut command = Command::new(exe_name);
    let output = command.output().await.unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    info!("{:?}", stdout);
    // info!("{:?}", std::str::from_utf8(&output.stderr));
    // info!("{:?}", output.status);

    Ok(ExecuteResponse {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
    })
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
