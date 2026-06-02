use std::{
    io::{self, IsTerminal},
    process::exit,
};

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
    } else if !io::stdin().is_terminal() {
        let lines: Result<Vec<String>, io::Error> = io::stdin().lines().collect();
        let lines = lines.unwrap();
        let mut pass = true;
        for p in &lines {
            if !std::fs::exists(p).unwrap() {
                eprintln!("{p:?} not exists");
                pass = false;
            }
        }
        if !pass {
            exit(1);
        }
        ctx.set_files(args.files).unwrap();
    } else {
        eprintln!("no file provided");
        exit(1);
    }
}
