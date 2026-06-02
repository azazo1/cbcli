use std::{
    io::{self, IsTerminal},
    path::Path,
    process::exit,
    time::Duration,
};

use clap::Parser;
use clipboard_rs::{Clipboard, ClipboardContext};

#[derive(Parser, Debug)]
struct AppArgs {
    #[arg(index = 1)]
    files: Vec<String>,
}

fn copy_files(files: &[impl AsRef<Path>]) {
    let ctx = ClipboardContext::new().unwrap();
    ctx.set_files(
        files
            .iter()
            .map(|f| {
                url::Url::from_file_path(std::path::absolute(f.as_ref()).unwrap())
                    .unwrap()
                    .as_str()
                    .to_owned()
            })
            .collect(),
    )
    .unwrap();
}

fn main() {
    let args = AppArgs::parse();
    if !args.files.is_empty() {
        copy_files(&args.files);
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
        copy_files(&lines);
    } else {
        eprintln!("no file provided");
        exit(1);
    }
    std::thread::sleep(Duration::from_millis(500));
}
