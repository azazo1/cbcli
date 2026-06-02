# cbcli

一个基于 macOS 剪贴板的文件复制粘贴小工具集, 当前包含:

- `pfcopy`: 把文件路径写入系统剪贴板
- `pfpaste`: 从系统剪贴板读取文件并复制或移动到目标目录

## 构建

```bash
cargo build
```

也可以直接运行单个命令:

```bash
cargo run --bin pfcopy -- <file>...
cargo run --bin pfpaste -- [options]
```

## pfcopy

把一个或多个文件放进系统剪贴板:

```bash
cargo run --bin pfcopy -- path/to/a path/to/b
```

也可以从标准输入逐行读取文件路径:

```bash
printf '%s\n' path/to/a path/to/b | cargo run --bin pfcopy
```

如果输入的任意路径不存在, 命令会报错并返回非零退出码.

## pfpaste

从系统剪贴板读取文件, 默认复制到当前目录:

```bash
cargo run --bin pfpaste
```

常用选项:

- `-C <dir>`: 指定目标目录
- `-m, --move`: 移动文件而不是复制文件
- `-w, --overwrite`: 允许覆盖已存在的目标文件
- `-n, --dry-run`: 只输出目标路径, 不执行实际写入

示例:

```bash
cargo run --bin pfpaste -- -C /tmp/out
cargo run --bin pfpaste -- -m -C /tmp/out
cargo run --bin pfpaste -- -w -C /tmp/out
cargo run --bin pfpaste -- --dry-run -C /tmp/out
```

如果目标文件已存在且未指定 `--overwrite`, 命令会报错并返回非零退出码.

## 说明

这个项目依赖系统剪贴板, 需要在可访问 macOS 图形会话剪贴板的环境里运行.
