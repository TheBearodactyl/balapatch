// #![allow(unused)]

use inquire::error::InquireResult;

mod balapatch;

#[tokio::main]
async fn main() -> InquireResult<()> {
    balapatch::tui::balapatch::balapatch().await
}
