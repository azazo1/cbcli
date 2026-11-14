use std::{
    io::{self, IsTerminal},
    path::Path,
    process::exit,
    time::Duration,
};

use clap::Parser;
use clipboard_rs::{Clipboard, ClipboardContext};

#[path = "../version.rs"]
mod version;

#[derive(Parser, Debug)]
#[command(name = "pfcopy", version = version::BUILD_VERSION)]
struct AppArgs {
    #[arg(index = 1)]
    files: Vec<String>,
}

fn copy_files(files: &[impl AsRef<Path>]) {
    let ctx = ClipboardContext::new().unwrap();
    // clipboard-rs 在 macOS 上自己把路径构造成 NSURL, 并过滤掉不存在的路径;
    // 传 file:// URL 时空格等字符会被百分号编码 (%20), 每个路径都被判为不存在,
    // 于是报 "no valid files". Windows 侧的 CF_HDROP 同样只接受裸路径.
    ctx.set_files(
        files
            .iter()
            .map(|f| {
                std::path::absolute(f.as_ref())
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
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
