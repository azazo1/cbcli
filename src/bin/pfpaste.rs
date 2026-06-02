use clap::Parser;
use clipboard_rs::{Clipboard, ClipboardContext};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Parser, Debug)]
struct AppArgs {
    #[arg(short = 'C', help = "粘贴的目录")]
    paste_path: Option<String>,
    #[arg(short = 'm', long = "move", help = "移动文件而不是复制文件")]
    mv: bool,
    #[arg(short = 'w', long = "overwrite", help = "允许覆盖写入")]
    overwrite: bool,
    #[arg(short = 'n', long = "dry-run", help = "查看写入文件情况")]
    dry_run: bool,
}

fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }

    fs::copy(from, to)?;
    fs::remove_file(from)?;
    Ok(())
}

fn main() {
    let AppArgs {
        paste_path,
        mv,
        overwrite,
        dry_run,
    } = AppArgs::parse();
    let paste_path = PathBuf::from(paste_path.unwrap_or(".".into()));
    let ctx = ClipboardContext::new().unwrap();
    let files = ctx
        .get_files()
        .unwrap()
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    let mut pass = true;
    for file in &files {
        let to = paste_path.join(file.file_name().unwrap());
        if to.exists() && !overwrite {
            eprintln!("{} exists and not allowed to overwrite.", to.display());
            pass = false;
        }
    }
    if !pass {
        exit(1);
    }

    for file in &files {
        let to = paste_path.join(file.file_name().unwrap());
        println!("{}", to.display());
        if !dry_run {
            if mv {
                move_file(file, &to).unwrap();
            } else {
                fs::copy(file, to).unwrap();
            }
        }
    }
}
