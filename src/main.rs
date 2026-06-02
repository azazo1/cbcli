use std::{io, path::PathBuf};

use clap::Parser;
use clipboard_rs::Clipboard;

#[derive(Parser, Debug)]
struct AppArgs {
    #[arg(index = 1)]
    files: Vec<String>,
}

fn main() {
    let args = AppArgs::parse();
    let ctx = clipboard_rs::ClipboardContext::new().unwrap();
    if !args.files.is_empty() {
        ctx.set_files(args.files).unwrap();
    } else {
        let lines: Result<Vec<String>, io::Error> = io::stdin().lines().collect();
        let lines = lines.unwrap();
        for p in &lines {
            if !std::fs::exists(p).unwrap() {
                panic!("{p:?} not exists");
            }
        }
        ctx.set_files(args.files).unwrap();
    }
}
