#![allow(unused)]

mod adb;
mod apk;
mod balatro;
mod cli;
mod patcher;
mod writer;
mod progress;
mod utils;
mod zipalign;

use std::rc::Rc;
use std::sync::Arc;
use anyhow::Context;
use clap::Parser;
use cli::program;
use tracing::{Level, debug, error, info};
use tracing_subscriber::fmt::format::FmtSpan;
use crate::progress::GLOBAL_MP;
use crate::writer::MultiProgressWriter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mp = Arc::new(progress::GLOBAL_MP.clone());
    
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(move || MultiProgressWriter::new(mp.clone()))
        .init();

    program::program().await
}
