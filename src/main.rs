#![allow(unused)]

mod adb;
mod apk;
mod balatro;
mod cli;
mod utils;
mod zipalign;

use anyhow::Context;
use clap::Parser;
use cli::program;

fn main() -> anyhow::Result<()> {
    program::program()
}