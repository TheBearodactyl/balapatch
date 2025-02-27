#![allow(unused, dead_code)]

use inquire::error::InquireResult;

mod balapatch;

#[tokio::main]
async fn main() -> InquireResult<()> {
    balapatch::tui::balapatch::balapatch().await
}
