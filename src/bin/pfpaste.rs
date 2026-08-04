use anyhow::{bail, Context, Result};
use clap::Parser;
use clipboard_rs::common::RustImage;
use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

/// 移动单个文件: 优先 rename (同分区瞬间完成), 失败则复制后删源.
fn move_file(from: &Path, to: &Path) -> Result<()> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    fs::copy(from, to).with_context(|| format!("复制 {} 失败", from.display()))?;
    fs::remove_file(from).with_context(|| format!("删除源 {} 失败", from.display()))?;
    Ok(())
}

/// 移动目录: 优先 rename, 跨分区失败时用 clonetree 克隆后再删除源.
fn move_dir(from: &Path, to: &Path) -> Result<()> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    clonetree::clone_tree(from, to, &clonetree::Options::new())
        .with_context(|| format!("克隆目录 {} 失败", from.display()))?;
    fs::remove_dir_all(from)
        .with_context(|| format!("删除源目录 {} 失败", from.display()))?;
    Ok(())
}

fn image_file_name() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("clipboard-{millis}.png")
}

fn main() -> Result<()> {
    let AppArgs {
        paste_path,
        mv,
        overwrite,
        dry_run,
    } = AppArgs::parse();
    let paste_path = PathBuf::from(paste_path.unwrap_or(".".into()));

    let ctx = ClipboardContext::new().map_err(|e| anyhow::anyhow!("无法访问系统剪贴板: {e}"))?;
    let files = if ctx.has(ContentFormat::Files) {
        ctx.get_files()
            .map_err(|e| anyhow::anyhow!("从剪贴板读取文件列表失败: {e}"))?
            .into_iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let image = if files.is_empty() && ctx.has(ContentFormat::Image) {
        Some(
            ctx.get_image()
                .map_err(|e| anyhow::anyhow!("从剪贴板读取图片失败: {e}"))?,
        )
    } else {
        None
    };

    if files.is_empty() && image.is_none() {
        bail!("剪贴板里没有文件或图片");
    }
    if image.is_some() && mv {
        bail!("图片来自剪贴板数据, 不能使用 --move");
    }
    let image_to = image.as_ref().map(|_| paste_path.join(image_file_name()));

    // 确保目标目录存在
    if !dry_run {
        fs::create_dir_all(&paste_path)
            .with_context(|| format!("创建目标目录 {} 失败", paste_path.display()))?;
    }

    let mut pass = true;
    for file in &files {
        let Some(name) = file.file_name() else {
            // file_name() 仅在路径以 .. 或 / 结尾时返回 None; 剪贴板里的 file:// 都是绝对路径文件, 实际不会触发
            eprintln!("无法确定 {} 的文件名", file.display());
            pass = false;
            continue;
        };
        let to = paste_path.join(name);
        if to.exists() && !overwrite {
            eprintln!("{} 已存在且未指定 --overwrite", to.display());
            pass = false;
        }
    }
    if !pass {
        std::process::exit(1);
    }

    if let Some(to) = &image_to
        && to.exists()
        && !overwrite
    {
        eprintln!("{} 已存在且未指定 --overwrite", to.display());
        std::process::exit(1);
    }

    for file in &files {
        let name = file.file_name().expect("已在上一轮校验通过");
        let to = paste_path.join(name);
        println!("{}", to.display());
        if dry_run {
            continue;
        }
        if to.exists() && overwrite {
            if to.is_dir() {
                fs::remove_dir_all(&to)
                    .with_context(|| format!("清空目标目录 {} 失败", to.display()))?;
            } else {
                fs::remove_file(&to)
                    .with_context(|| format!("删除目标文件 {} 失败", to.display()))?;
            }
        }
        if mv {
            if file.is_dir() {
                move_dir(file, &to)?;
            } else {
                move_file(file, &to)?;
            }
        } else if file.is_dir() {
            clonetree::clone_tree(file, &to, &clonetree::Options::new())
                .with_context(|| format!("克隆目录 {} 失败", file.display()))?;
        } else {
            fs::copy(file, &to).with_context(|| format!("复制文件 {} 失败", file.display()))?;
        }
    }

    if let Some(image) = image {
        let to = image_to.expect("image target computed above");
        println!("{}", to.display());
        if dry_run {
            return Ok(());
        }
        if to.exists() && overwrite {
            if to.is_dir() {
                fs::remove_dir_all(&to)
                    .with_context(|| format!("清空目标目录 {} 失败", to.display()))?;
            } else {
                fs::remove_file(&to)
                    .with_context(|| format!("删除目标文件 {} 失败", to.display()))?;
            }
        }
        image
            .save_to_path(to.to_string_lossy().as_ref())
            .map_err(|e| anyhow::anyhow!("保存图片 {} 失败: {e}", to.display()))?;
    }

    Ok(())
}
