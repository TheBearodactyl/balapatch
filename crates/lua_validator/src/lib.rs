//! This does not need to be a whole separate crate but whatever lmao

use anyhow::anyhow;
use rslua::lexer::Lexer;

#[derive(Debug)]
pub struct LVal {
    pub src: String,
}

impl LVal {
    pub fn new(src: String) -> Self {
        Self { src }
    }

    pub fn print_src(&self) {
        println!("{}", self.src);
    }

    pub fn validate(&self) -> anyhow::Result<(), String> {
        let mut lexer = Lexer::default();

        if let Ok(tokens) = lexer.run(&self.src) {
            Ok(())
        } else {
            Err(anyhow!("Couldn't validate provided Lua code").to_string())
        }
    }
}
