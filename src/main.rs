#![allow(unused)]

mod balapatch;

use balapatch::prelude::*;

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
