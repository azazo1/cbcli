[private]
default:
    @just --list

# 根据当前平台生成发布产物, 一个归档里含 pfcopy 与 pfpaste.
dist:
    uv run --no-project python scripts/dist.py
