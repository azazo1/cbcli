#!/usr/bin/env python3

# 生成当前平台的发布产物 (Rust), 位于项目的 scripts/dist.py.
# 需要同时复制 scripts/archive.py, scripts/build-version.py, scripts/release_common.py,
# scripts/output_encoding.py 与项目根的 build.rs.
#
# 这个项目一个归档里放两个可执行文件, 所以 BINARY_NAMES 是列表;
# 判定条件与版本解析都沿用各语言共用的 release_common.

import json
import os
import shutil

import release_common as rc

# 只改下面几个常量:
PROJECT_NAME = "cbcli"
# 读包版本用的 crate 名, 与产物名不同时改这里.
PACKAGE_NAME = "cbcli"
# 要打进同一个归档的可执行文件名, 不带扩展名.
BINARY_NAMES = ["pfcopy", "pfpaste"]
# 冒烟检查用的参数, 要求二进制能报出版本号.
SMOKE_ARGS = ["--version"]
# 需要交叉编译时设置, 例如 aarch64-unknown-linux-musl;
# 交叉编译的产物无法本机执行, 只能做文件格式检查与架构核对,
# 并用 TARGET_PLATFORM / TARGET_ARCH 命名.
RUST_TARGET = os.environ.get("RUST_TARGET", "")


def package_version_fallback():
    """仓库还没有版本 tag 时, 用 cargo metadata 里的包版本兜底; 读不到就返回 None."""
    metadata = rc.try_run(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"])
    if not metadata:
        return None
    try:
        for package in json.loads(metadata)["packages"]:
            if package.get("name") == PACKAGE_NAME:
                return package.get("version")
    except (KeyError, ValueError):
        return None
    return None


def assert_runner_expectation():
    """CI 里用 EXPECT_PLATFORM / EXPECT_ARCH 断言矩阵标签与真实运行器一致.

    runner 标签与真实架构不符时, 悄悄产出一个命名错误的包比直接失败更糟.
    """
    actual_platform = rc.detect_platform()
    actual_arch = rc.detect_arch()
    expected_platform = os.environ.get("EXPECT_PLATFORM", "").strip()
    expected_arch = os.environ.get("EXPECT_ARCH", "").strip()

    if expected_platform and expected_platform != actual_platform:
        rc.die(f"运行器平台是 {actual_platform}, 与期望的 {expected_platform} 不一致")
    if expected_arch and expected_arch != actual_arch:
        rc.die(f"运行器架构是 {actual_arch}, 与期望的 {expected_arch} 不一致")


def main():
    rc.configure_output_encoding()
    rc.repo_root()
    assert_runner_expectation()

    target_platform = rc.target_platform()
    target_arch = rc.target_arch()
    display_version, archive_version = rc.resolve_versions(
        rc.script_path("build-version.py"), package_version_fallback
    )
    print(f"构建 {PROJECT_NAME} {display_version} ({target_platform}-{target_arch})")

    # build.rs 只在设置了 PROJECT_BUILD_VERSION 时注入版本, 所以这里显式传给 cargo,
    # 否则直接跑 scripts/dist.py 时产物里还是 dev-build.
    cargo_env = os.environ.copy()
    cargo_env["PROJECT_BUILD_VERSION"] = display_version
    cargo_args = ["cargo", "build", "--release", "--locked"]
    if RUST_TARGET:
        cargo_args += ["--target", RUST_TARGET]
    rc.run(cargo_args, env=cargo_env, capture=False)

    suffix = ".exe" if target_platform == "windows" else ""
    build_dir = f"target/{RUST_TARGET}/release" if RUST_TARGET else "target/release"
    staging = rc.stage_dir()
    binaries = [name + suffix for name in BINARY_NAMES]
    for binary in binaries:
        shutil.copy(os.path.join(build_dir, binary), os.path.join(staging, binary))

    if RUST_TARGET:
        print("交叉编译产物无法本机执行, 只检查文件格式与架构")
        for binary in binaries:
            path = os.path.join(staging, binary)
            rc.check_binary_format(path, target_platform)
            rc.check_binary_arch(path, target_platform, target_arch)
            rc.check_version_string(path, display_version)
    else:
        # 逐个跑冒烟检查: 只验一个会漏掉另一个 bin 的版本注入.
        for binary in binaries:
            rc.smoke_check([os.path.join(staging, binary)], SMOKE_ARGS, display_version)

    rc.archive(
        staging,
        binaries,
        PROJECT_NAME,
        archive_version,
        target=(target_platform, target_arch),
    )


if __name__ == "__main__":
    main()
