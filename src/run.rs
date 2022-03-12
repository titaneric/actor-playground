use actix_web::{post, web, HttpResponse, Responder, Result};
use futures::future::poll_fn;
use futures::StreamExt;
use log::info;
use runner::runner_server::{Runner, RunnerServer};
use runner::{ExecuteRequest, ExecuteResponse};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::pin::Pin;
use std::{
    fs::{set_permissions, File},
    io::Write,
    path::{Path, PathBuf},
};
use tempdir::TempDir;
use tokio::fs::File as TokioFile;

use bytes::{Buf, Bytes};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio_util::io::poll_write_buf;
use tonic::{
    transport::Server, Request as TonicRequest, Response as TonicResponse, Status, Streaming,
};

pub mod runner {
    tonic::include_proto!("runner");
}

#[derive(Debug, Default)]
pub struct RunnerImpl {}

#[tonic::async_trait]
impl Runner for RunnerImpl {
    async fn execute(
        &self,
        request: TonicRequest<Streaming<ExecuteRequest>>, // Accept request of type HelloRequest
    ) -> Result<TonicResponse<ExecuteResponse>, Status> {
        // Return an instance of type HelloReply
        let mut stream = request.into_inner();
        let handle = tokio::spawn(async move {
            let tmp_dir = TempDir::new("rust-run").unwrap();
            let filename = tmp_dir.into_path().join("playground");
            let mut f = tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o755)
                .open(&filename)
                .await
                .unwrap();
            while let Some(req) = stream.next().await {
                // poll_fn(|cx| poll_write_buf(Pin::new(&mut f), cx, &mut Bytes::from( &req.as_ref().unwrap().binary.into())))
                //     .await?;
                info!(
                    "Got a binary size is: {:?}",
                    req.as_ref().unwrap().binary.len()
                );
                f.write_all(&req.unwrap().binary).await.unwrap();
            }
            return filename;
        });
        let filename = handle.await.unwrap();
        let reply = run_handler(filename).await.unwrap();
        Ok(TonicResponse::new(reply)) // Send back our formatted greeting
    }
}

async fn run_handler(exe_name: PathBuf) -> Result<ExecuteResponse> {
    // let mut bytes = web::BytesMut::new();
    // while let Some(item) = body.next().await {
    //     let item = item.unwrap();
    //     bytes.extend_from_slice(&item);
    // }
    // let run_req: RunReq = serde_json::from_slice(&bytes).unwrap();

    // info!("Chunk: {:?}", run_req.built_binary.len());
    // let filename = "runner";
    // let exe_name = write_byte_stream(built_binary, filename);
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
async fn write_byte_stream(buf: &[u8], filename: &str) -> TokioFile {
    let tmp_dir = TempDir::new("rust-run").unwrap();
    let filename = tmp_dir.into_path().join(filename);
    let mut f = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o755)
        .open(&filename)
        .await
        .unwrap();
    // f.write_all(buf).unwrap();
    // info!("{:#o}", f.metadata().unwrap().permissions().mode());
    f
}
