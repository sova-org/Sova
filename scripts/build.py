#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["rich>=13.0", "questionary>=2.0"]
# ///
"""Sova release builder."""

from __future__ import annotations

import argparse
import hashlib
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import tomllib
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path

from rich.console import Console, Group
from rich.markup import escape
from rich.live import Live
from rich.panel import Panel
from rich.progress_bar import ProgressBar
from rich.table import Table
from rich.text import Text

import questionary
from questionary import Style as QStyle

console = Console()

PROMPT_STYLE = QStyle([
    ("qmark", "fg:cyan bold"),
    ("question", "fg:white bold"),
    ("pointer", "fg:cyan bold"),
    ("highlighted", "fg:cyan bold"),
    ("selected", "fg:green"),
    ("instruction", "fg:white"),
    ("text", "fg:white"),
])

# ---------------------------------------------------------------------------
# Build progress tracking (shared between threads)
# ---------------------------------------------------------------------------

_progress_lock = threading.Lock()
_build_progress: dict[str, tuple[str, int]] = {}  # alias -> (phase, step)
_build_logs: list[tuple[str, str]] = []  # (alias, line)


def _update_phase(alias: str, phase: str, step: int) -> None:
    with _progress_lock:
        _build_progress[alias] = (phase, step)


class BuildLog(list):
    """A list that also feeds lines into the shared log buffer."""

    def __init__(self, alias: str):
        super().__init__()
        self._alias = alias

    def append(self, line: str) -> None:
        super().append(line)
        with _progress_lock:
            _build_logs.append((self._alias, line))

# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------

OUT = "releases"

# Binary names (as produced by cargo)
FRONTEND_BIN = "sova-frontend"
SERVER_BIN = "sova_server"

# Package names (for cargo -p)
FRONTEND_PKG = "sova-desktop"
SERVER_PKG = "sova-server"


@dataclass(frozen=True)
class Platform:
    triple: str
    label: str
    alias: str
    os: str
    arch: str
    cross: bool
    native: bool


def _parse_triple(triple: str) -> Platform:
    """Derive a full Platform from a Rust target triple."""
    parts = triple.split("-")
    arch = parts[0]
    if "apple" in triple:
        os_name, cross = "macos", False
    elif "linux" in triple:
        os_name, cross = "linux", True
    elif "windows" in triple:
        os_name, cross = "windows", True
    else:
        raise ValueError(f"Unknown OS in triple: {triple}")
    host_arch = platform.machine()
    host_triple_arch = "aarch64" if host_arch == "arm64" else host_arch
    native = (os_name == "macos" and arch == host_triple_arch and platform.system() == "Darwin") \
          or (os_name == "linux" and arch == host_triple_arch and platform.system() == "Linux")
    mode = "native" if native else "cross" if cross else "native"
    alias_arch = "arm64" if (os_name == "macos" and arch == "aarch64") else arch
    alias = f"{os_name}-{alias_arch}"
    label = f"{'macOS' if os_name == 'macos' else os_name.capitalize()} {arch} ({mode})"
    return Platform(triple, label, alias, os_name, arch, cross, native)


def load_platforms(root: Path) -> list[Platform]:
    """Load platform definitions from scripts/platforms.toml."""
    with open(root / "scripts" / "platforms.toml", "rb") as f:
        data = tomllib.load(f)
    return [_parse_triple(t) for t in data["triples"]]


@dataclass
class BuildConfig:
    frontend: bool = True
    server: bool = True


@dataclass
class PlatformResult:
    platform: Platform
    success: bool
    elapsed: float
    artifacts: list[str] = field(default_factory=list)
    log_lines: list[str] = field(default_factory=list)
    error: str | None = None


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def get_version(root: Path) -> str:
    with open(root / "desktop" / "Cargo.toml", "rb") as f:
        cargo = tomllib.load(f)
    version = cargo["package"]["version"]
    if isinstance(version, dict) and version.get("workspace"):
        with open(root / "Cargo.toml", "rb") as f:
            workspace = tomllib.load(f)
        return workspace["workspace"]["package"]["version"]
    return version


def builder_for(p: Platform) -> str:
    if p.os == "windows" and p.cross:
        return "cargo-xwin"
    return "cross" if p.cross else "cargo"


def release_dir(root: Path, p: Platform) -> Path:
    if p.native:
        return root / "target" / "release"
    return root / "target" / p.triple / "release"


def target_flags(p: Platform) -> list[str]:
    if p.native:
        return []
    return ["--target", p.triple]


def suffix_for(p: Platform) -> str:
    return ".exe" if p.os == "windows" else ""


def run_cmd(
    cmd: list[str], log: list[str], env: dict[str, str] | None = None,
    input: str | None = None, cwd: Path | None = None,
) -> None:
    log.append(f"  $ {' '.join(cmd)}")
    merged_env = {**os.environ, **(env or {})}
    proc = subprocess.Popen(
        cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        text=True, env=merged_env,
        stdin=subprocess.PIPE if input else subprocess.DEVNULL,
        cwd=cwd,
    )
    if input:
        proc.stdin.write(input)
        proc.stdin.close()
    for line in proc.stdout:
        log.append(f"    {line.rstrip()}")
    rc = proc.wait()
    if rc != 0:
        raise subprocess.CalledProcessError(rc, cmd)


# ---------------------------------------------------------------------------
# Build functions
# ---------------------------------------------------------------------------

def _find_llvm_prefix() -> str | None:
    """Find Homebrew LLVM installation path."""
    try:
        result = subprocess.run(
            ["brew", "--prefix", "llvm"], capture_output=True, text=True,
        )
        if result.returncode == 0:
            return result.stdout.strip()
    except FileNotFoundError:
        pass
    return None


def _asio_sdk_valid(path: Path) -> bool:
    """Check that an ASIO SDK directory contains the required headers."""
    return (path / "host" / "asiodrivers.h").exists()


def _ensure_asio_sdk(root: Path, log: list[str]) -> Path:
    """Ensure a complete ASIO SDK is available, downloading if needed."""
    # Check project cache first
    cache_sdk = root / ".cache" / "asio-sdk"
    if cache_sdk.exists() and _asio_sdk_valid(cache_sdk):
        log.append(f"  ASIO SDK found at {cache_sdk}")
        return cache_sdk

    # Check temp dir (where asio-sys auto-downloads)
    temp_sdk = Path(tempfile.gettempdir()) / "asio_sdk"
    if temp_sdk.exists() and _asio_sdk_valid(temp_sdk):
        log.append(f"  ASIO SDK found at {temp_sdk}")
        return temp_sdk

    # Download to project cache
    log.append("  Downloading ASIO SDK...")
    if cache_sdk.exists():
        shutil.rmtree(cache_sdk)
    cache_sdk.parent.mkdir(parents=True, exist_ok=True)
    archive = cache_sdk.parent / "asio-sdk.zip"
    run_cmd(["curl", "-fSL", "https://www.steinberg.net/asiosdk", "-o", str(archive)], log)
    run_cmd(["unzip", "-q", "-o", str(archive), "-d", str(cache_sdk.parent)], log)
    # The archive extracts to a subdirectory — find and rename it
    for child in cache_sdk.parent.iterdir():
        if child.is_dir() and child.name.lower().startswith("asio") and child != cache_sdk:
            child.rename(cache_sdk)
            break
    archive.unlink(missing_ok=True)

    if not _asio_sdk_valid(cache_sdk):
        raise RuntimeError(f"Downloaded ASIO SDK is incomplete (missing host/asiodrivers.h)")
    log.append(f"  ASIO SDK ready at {cache_sdk}")
    return cache_sdk


def _cross_env(p: Platform, root: Path | None = None, log: list[str] | None = None) -> dict[str, str] | None:
    env = {}
    if p.os == "macos":
        env["MACOSX_DEPLOYMENT_TARGET"] = "12.0"
    if p.os == "windows" and p.cross:
        env["VCINSTALLDIR"] = "/dummy"
        llvm_prefix = _find_llvm_prefix()
        if llvm_prefix:
            env["LIBCLANG_PATH"] = f"{llvm_prefix}/lib"
        if root and log is not None:
            asio_dir = _ensure_asio_sdk(root, log)
            env["CPAL_ASIO_DIR"] = str(asio_dir)
    return env or None


def _platform_features(p: Platform, pkg: str) -> list[str]:
    if p.os == "windows":
        return ["--features", "asio"]
    return []


def build_binary(root: Path, p: Platform, log: list[str], pkg: str, bin_name: str) -> None:
    features = _platform_features(p, pkg)
    cmd = [builder_for(p), "build", "--release", *target_flags(p), "-p", pkg, *features]
    log.append(f"  Building: {bin_name}")
    run_cmd(cmd, log, env=_cross_env(p, root, log), cwd=root)


# ---------------------------------------------------------------------------
# Packaging: macOS .app bundle
# ---------------------------------------------------------------------------


def bundle_app(root: Path, p: Platform, log: list[str]) -> Path:
    """Create a macOS .app bundle manually."""
    rd = release_dir(root, p)
    binary = rd / FRONTEND_BIN
    if not binary.exists():
        raise FileNotFoundError(f"Binary not found: {binary}")

    app_dir = rd / "bundle" / "osx" / "Sova.app"
    contents = app_dir / "Contents"
    if app_dir.exists():
        shutil.rmtree(app_dir)
    (contents / "MacOS").mkdir(parents=True)
    (contents / "Resources").mkdir(parents=True)

    shutil.copy2(binary, contents / "MacOS" / FRONTEND_BIN)

    icon = root / "desktop" / "assets" / "Sova.icns"
    if icon.exists():
        shutil.copy2(icon, contents / "Resources" / "Sova.icns")

    version = get_version(root)
    plist = f"""\
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Sova</string>
    <key>CFBundleDisplayName</key>
    <string>Sova</string>
    <key>CFBundleIdentifier</key>
    <string>com.sova.sova</string>
    <key>CFBundleVersion</key>
    <string>{version}</string>
    <key>CFBundleShortVersionString</key>
    <string>{version}</string>
    <key>CFBundleExecutable</key>
    <string>{FRONTEND_BIN}</string>
    <key>CFBundleIconFile</key>
    <string>Sova.icns</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.music</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright (c) 2025 Raphaël Forment</string>
    <key>NSMicrophoneUsageDescription</key>
    <string>Sova needs microphone access for audio input.</string>
</dict>
</plist>"""
    (contents / "Info.plist").write_text(plist)

    log.append(f"    APP -> {app_dir}")
    return app_dir


# ---------------------------------------------------------------------------
# Packaging: DMG
# ---------------------------------------------------------------------------


def make_dmg(root: Path, app_path: Path, arch: str, output_dir: Path, log: list[str]) -> str | None:
    log.append(f"  Building DMG for {app_path.name}")

    binary = app_path / "Contents" / "MacOS" / FRONTEND_BIN
    result = subprocess.run(["lipo", "-info", str(binary)], capture_output=True, text=True)
    if result.returncode != 0:
        log.append(f"    ERROR: lipo failed on {binary}")
        return None

    lipo_out = result.stdout.strip()
    if "Architectures in the fat file" in lipo_out:
        dmg_arch = "universal"
    else:
        raw_arch = lipo_out.split()[-1]
        dmg_arch = "aarch64" if raw_arch == "arm64" else raw_arch

    staging = Path(tempfile.mkdtemp())
    try:
        shutil.copytree(app_path, staging / "Sova.app")
        (staging / "Applications").symlink_to("/Applications")

        dmg_name = f"Sova-{dmg_arch}.dmg"
        output_dir.mkdir(parents=True, exist_ok=True)
        dmg_path = output_dir / dmg_name

        run_cmd([
            "hdiutil", "create",
            "-volname", "Sova",
            "-srcfolder", str(staging),
            "-ov", "-format", "UDZO",
            str(dmg_path),
        ], log)
        log.append(f"    DMG -> {dmg_path}")
        return str(dmg_path)
    finally:
        shutil.rmtree(staging, ignore_errors=True)


# ---------------------------------------------------------------------------
# Packaging: AppImage
# ---------------------------------------------------------------------------

APPRUN_SCRIPT = """\
#!/bin/sh
SELF="$(readlink -f "$0")"
HERE="$(dirname "$SELF")"
exec "$HERE/usr/bin/{bin}" "$@"
"""


def _build_appdir(root: Path, binary: Path, appdir: Path, bin_name: str) -> None:
    (appdir / "usr" / "bin").mkdir(parents=True)
    shutil.copy2(binary, appdir / "usr" / "bin" / bin_name)
    (appdir / "usr" / "bin" / bin_name).chmod(0o755)

    icons = appdir / "usr" / "share" / "icons" / "hicolor" / "512x512" / "apps"
    icons.mkdir(parents=True)
    shutil.copy2(root / "desktop" / "assets" / "icon.png", icons / "sova.png")
    shutil.copy2(root / "desktop" / "assets" / "sova.desktop", appdir / "sova.desktop")

    apprun = appdir / "AppRun"
    apprun.write_text(APPRUN_SCRIPT.format(bin=bin_name))
    apprun.chmod(0o755)

    (appdir / "sova.png").symlink_to("usr/share/icons/hicolor/512x512/apps/sova.png")


def _download_if_missing(url: str, dest: Path, log: list[str]) -> None:
    if dest.exists():
        return
    dest.parent.mkdir(parents=True, exist_ok=True)
    log.append(f"  Downloading {url}")
    run_cmd(["curl", "-fSL", url, "-o", str(dest)], log)


def _make_appimage_native(root: Path, binary: Path, arch: str, output_dir: Path, log: list[str]) -> str:
    cache = root / ".cache"
    runtime = cache / f"runtime-{arch}"
    _download_if_missing(
        f"https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-{arch}",
        runtime, log,
    )

    linuxdeploy = cache / f"linuxdeploy-{arch}.AppImage"
    _download_if_missing(
        f"https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-{arch}.AppImage",
        linuxdeploy, log,
    )
    linuxdeploy.chmod(0o755)

    bin_name = binary.name
    appdir = Path(tempfile.mkdtemp()) / "AppDir"
    _build_appdir(root, binary, appdir, bin_name)

    env = {"ARCH": arch, "LDAI_RUNTIME_FILE": str(runtime)}
    run_cmd([
        str(linuxdeploy),
        "--appimage-extract-and-run",
        "--appdir", str(appdir),
        "--desktop-file", str(appdir / "sova.desktop"),
        "--icon-file", str(appdir / "usr" / "share" / "icons" / "hicolor" / "512x512" / "apps" / "sova.png"),
        "--output", "appimage",
    ], log, env=env, cwd=root)

    candidates = sorted(root.glob("*.AppImage"), key=lambda p: p.stat().st_mtime, reverse=True)
    if not candidates:
        raise FileNotFoundError("No AppImage produced by linuxdeploy")

    output_dir.mkdir(parents=True, exist_ok=True)
    final = output_dir / f"{bin_name}-linux-{arch}.AppImage"
    shutil.move(str(candidates[0]), final)
    log.append(f"    AppImage -> {final}")
    return str(final)


def _make_appimage_docker(root: Path, binary: Path, arch: str, output_dir: Path, log: list[str]) -> str:
    cache = root / ".cache"
    runtime = cache / f"runtime-{arch}"
    _download_if_missing(
        f"https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-{arch}",
        runtime, log,
    )

    bin_name = binary.name
    appdir = Path(tempfile.mkdtemp()) / "AppDir"
    _build_appdir(root, binary, appdir, bin_name)

    docker_platform = "linux/amd64" if arch == "x86_64" else "linux/arm64"
    image_tag = f"sova-appimage-{arch}"

    log.append(f"  Building Docker image {image_tag} ({docker_platform})")
    dockerfile = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y --no-install-recommends squashfs-tools && rm -rf /var/lib/apt/lists/*\n"
    run_cmd([
        "docker", "build", "--platform", docker_platform, "-q", "-t", image_tag, "-",
    ], log, input=dockerfile)

    squashfs = cache / f"appimage-{arch}.squashfs"
    run_cmd([
        "docker", "run", "--rm", "--platform", docker_platform,
        "-v", f"{appdir}:/appdir:ro",
        "-v", f"{cache}:/cache",
        image_tag,
        "mksquashfs", "/appdir", f"/cache/appimage-{arch}.squashfs",
        "-root-owned", "-noappend", "-comp", "gzip", "-no-progress",
    ], log)

    output_dir.mkdir(parents=True, exist_ok=True)
    final = output_dir / f"{bin_name}-linux-{arch}.AppImage"
    with open(final, "wb") as out_f:
        out_f.write(runtime.read_bytes())
        out_f.write(squashfs.read_bytes())
    final.chmod(0o755)
    squashfs.unlink(missing_ok=True)
    log.append(f"    AppImage -> {final}")
    return str(final)


def make_appimage(root: Path, binary: Path, arch: str, output_dir: Path, log: list[str]) -> str:
    log.append(f"  Building AppImage for {binary.name} ({arch})")
    host_arch = platform.machine()
    if host_arch == arch and platform.system() == "Linux":
        return _make_appimage_native(root, binary, arch, output_dir, log)
    return _make_appimage_docker(root, binary, arch, output_dir, log)


# ---------------------------------------------------------------------------
# Artifact copying & packaging dispatch
# ---------------------------------------------------------------------------


def copy_artifacts(root: Path, p: Platform, config: BuildConfig, log: list[str]) -> list[str]:
    rd = release_dir(root, p)
    out = root / OUT
    sx = suffix_for(p)
    artifacts: list[str] = []

    if config.frontend:
        src = rd / f"{FRONTEND_BIN}{sx}"
        dst = out / f"{FRONTEND_BIN}-{p.os}-{p.arch}{sx}"
        shutil.copy2(src, dst)
        log.append(f"    {FRONTEND_BIN} -> {dst}")
        artifacts.append(str(dst))

        if p.os == "macos":
            app_src = rd / "bundle" / "osx" / "Sova.app"
            if not app_src.is_dir():
                raise FileNotFoundError(f".app bundle not found at {app_src}")
            app_dst = out / f"Sova-{p.arch}.app"
            if app_dst.exists():
                shutil.rmtree(app_dst)
            shutil.copytree(app_src, app_dst)
            log.append(f"    Sova.app -> {app_dst}")
            artifacts.append(str(app_dst))

            dmg = make_dmg(root, app_dst, p.arch, out, log)
            if dmg:
                artifacts.append(dmg)

        if p.os == "linux":
            ai = make_appimage(root, rd / FRONTEND_BIN, p.arch, out, log)
            artifacts.append(ai)

    if config.server:
        src = rd / f"{SERVER_BIN}{sx}"
        dst = out / f"{SERVER_BIN}-{p.os}-{p.arch}{sx}"
        shutil.copy2(src, dst)
        log.append(f"    {SERVER_BIN} -> {dst}")
        artifacts.append(str(dst))

    return artifacts


# ---------------------------------------------------------------------------
# Per-platform orchestration
# ---------------------------------------------------------------------------


def _count_phases(p: Platform, config: BuildConfig) -> int:
    n = 0
    if config.frontend:
        n += 1  # compile frontend
        if p.os == "macos":
            n += 1  # bundle .app
    if config.server:
        n += 1  # compile server
    n += 1  # copy artifacts / packaging
    return n


def build_platform(root: Path, p: Platform, config: BuildConfig) -> PlatformResult:
    log = BuildLog(p.alias)
    t0 = time.monotonic()
    step = 0
    try:
        if config.frontend:
            _update_phase(p.alias, "compiling frontend", step)
            build_binary(root, p, log, FRONTEND_PKG, FRONTEND_BIN)
            step += 1
            if p.os == "macos":
                _update_phase(p.alias, "bundling .app", step)
                bundle_app(root, p, log)
                step += 1

        if config.server:
            _update_phase(p.alias, "compiling server", step)
            build_binary(root, p, log, SERVER_PKG, SERVER_BIN)
            step += 1

        _update_phase(p.alias, "packaging", step)
        log.append("  Copying artifacts...")
        artifacts = copy_artifacts(root, p, config, log)

        elapsed = time.monotonic() - t0
        return PlatformResult(p, True, elapsed, artifacts, log)

    except Exception as e:
        elapsed = time.monotonic() - t0
        log.append(f"  ERROR: {e}")
        return PlatformResult(p, False, elapsed, [], log, str(e))


def _build_display(
    platforms: list[Platform],
    config: BuildConfig,
    completed: dict[str, PlatformResult],
    start_times: dict[str, float],
    log_max_lines: int,
) -> Group:
    table = Table(padding=(0, 1), expand=True, show_edge=False)
    table.add_column("Platform", style="cyan", no_wrap=True)
    table.add_column("Phase", no_wrap=True)
    table.add_column("Progress", no_wrap=True)
    table.add_column("Time", justify="right", no_wrap=True)

    with _progress_lock:
        progress_snapshot = dict(_build_progress)

    for p in platforms:
        alias = p.alias
        total = _count_phases(p, config)

        if alias in completed:
            r = completed[alias]
            if r.success:
                n = len(r.artifacts)
                phase = Text(f"OK ({n} artifacts)", style="green")
                bar = ProgressBar(total=total, completed=total, width=20, complete_style="green")
            else:
                phase = Text("FAIL", style="red")
                bar = ProgressBar(total=total, completed=total, width=20, complete_style="red")
            elapsed = f"{r.elapsed:.0f}s"
        elif alias in progress_snapshot:
            ph, step = progress_snapshot[alias]
            phase = Text(ph, style="yellow")
            bar = ProgressBar(total=total, completed=step, width=20)
            elapsed = f"{time.monotonic() - start_times.get(alias, time.monotonic()):.0f}s"
        else:
            phase = Text("waiting", style="dim")
            bar = ProgressBar(total=total, completed=0, width=20)
            elapsed = ""

        table.add_row(p.label, phase, bar, elapsed)

    with _progress_lock:
        recent = _build_logs[-log_max_lines:]

    if recent:
        lines: list[str] = []
        for alias, line in recent:
            lines.append(f"[dim]{alias}[/] {escape(line.rstrip())}")
        log_text = Text("\n") + Text.from_markup("\n".join(lines))
    else:
        log_text = Text("\nwaiting for output...", style="dim")

    return Group(table, log_text)


def _prebuild_cross_images(root: Path, platforms: list[Platform]) -> None:
    """Pre-build Docker images for cross-compiled targets in parallel."""
    cross_platforms = [p for p in platforms if p.cross and p.os != "windows"]
    if not cross_platforms:
        return

    cross_toml = root / "Cross.toml"
    if not cross_toml.exists():
        return

    with open(cross_toml, "rb") as f:
        cross_config = tomllib.load(f)

    def build_image(p: Platform) -> None:
        target_cfg = cross_config.get("target", {}).get(p.triple, {})
        dockerfile = target_cfg.get("dockerfile")
        if not dockerfile:
            return
        tag = f"cross-custom-{p.triple}:local"
        try:
            subprocess.run(
                ["docker", "build", "-t", tag, "-f", str(root / dockerfile), str(root)],
                capture_output=True, timeout=600,
            )
        except Exception:
            pass  # non-critical, cross will build if needed

    console.print("[dim]Pre-building cross-compilation Docker images...[/]")
    with ThreadPoolExecutor(max_workers=len(cross_platforms)) as pool:
        pool.map(build_image, cross_platforms)


def _build_native_sequential(
    root: Path,
    native_platforms: list[Platform],
    config: BuildConfig,
    completed: dict[str, PlatformResult],
    start_times: dict[str, float],
) -> list[PlatformResult]:
    """Build native platforms sequentially (they share the cargo target/ lock)."""
    native_results = []
    for p in native_platforms:
        start_times[p.alias] = time.monotonic()
        r = build_platform(root, p, config)
        completed[r.platform.alias] = r
        native_results.append(r)
    return native_results


def run_builds(
    root: Path, platforms: list[Platform], config: BuildConfig, version: str, verbose: bool = False,
) -> list[PlatformResult]:
    (root / OUT).mkdir(parents=True, exist_ok=True)

    _build_progress.clear()
    _build_logs.clear()
    for p in platforms:
        _update_phase(p.alias, "waiting", 0)

    # Docker-isolated cross builds (Linux only — each in its own container)
    docker_cross = [p for p in platforms if p.cross and p.os != "windows"]
    # Local builds share cargo lock — run sequentially
    local_builds = [p for p in platforms if not p.cross or p.os == "windows"]

    results: list[PlatformResult] = []
    completed: dict[str, PlatformResult] = {}
    start_times: dict[str, float] = {p.alias: time.monotonic() for p in platforms}

    term_height = console.size.height
    log_max_lines = max(term_height - len(platforms) - 10, 5)

    def make_display() -> Group:
        return _build_display(platforms, config, completed, start_times, log_max_lines)

    with Live(make_display(), console=console, refresh_per_second=4) as live:
        with ThreadPoolExecutor(max_workers=max(len(docker_cross) + 1, 1)) as pool:
            futures = {}

            if local_builds:
                f = pool.submit(
                    _build_native_sequential, root, local_builds, config,
                    completed, start_times,
                )
                futures[f] = "native"

            for p in docker_cross:
                f = pool.submit(build_platform, root, p, config)
                futures[f] = "cross"

            pending = set(futures.keys())
            while pending:
                done = {f for f in pending if f.done()}
                for f in done:
                    tag = futures[f]
                    if tag == "native":
                        results.extend(f.result())
                    else:
                        results.append(f.result())
                    pending.discard(f)
                live.update(make_display())
                if pending:
                    time.sleep(0.25)

    for r in results:
        _print_platform_log(r, verbose)

    _write_build_log(root / OUT / "build.log", results)

    return results


def _write_build_log(path: Path, results: list[PlatformResult]) -> None:
    with open(path, "w") as f:
        for r in results:
            status = "OK" if r.success else "FAIL"
            f.write(f"=== {r.platform.label} [{status}] {r.elapsed:.1f}s ===\n")
            for line in r.log_lines:
                f.write(line + "\n")
            f.write("\n")
    console.print(f"  [dim]Build log written to {path}[/]")


_FAILURE_LOG_TAIL = 50


def _print_platform_log(r: PlatformResult, verbose: bool = False) -> None:
    if r.success and not verbose:
        return
    style = "green" if r.success else "red"
    status = "OK" if r.success else "FAIL"

    lines = r.log_lines or []
    if not r.success and not verbose and len(lines) > _FAILURE_LOG_TAIL:
        lines = [f"  ... ({len(r.log_lines) - _FAILURE_LOG_TAIL} lines omitted, use --verbose for full output)"] + lines[-_FAILURE_LOG_TAIL:]

    console.print(Panel(
        "\n".join(escape(l) for l in lines) if lines else "[dim]no output[/]",
        title=f"{r.platform.label} [{status}] {r.elapsed:.1f}s",
        border_style=style,
    ))


# ---------------------------------------------------------------------------
# CLI & interactive mode
# ---------------------------------------------------------------------------


def _ask_or_exit(prompt) -> any:
    result = prompt.ask()
    if result is None:
        sys.exit(0)
    return result


def prompt_interactive(
    all_platforms: list[Platform], alias_map: dict[str, Platform],
) -> tuple[list[Platform], BuildConfig]:
    plat_choices = [questionary.Choice(p.label, value=p.alias, checked=p.native) for p in all_platforms]
    selected_aliases = _ask_or_exit(questionary.checkbox(
        "Platforms:", choices=plat_choices, style=PROMPT_STYLE,
    ))
    if not selected_aliases:
        selected_aliases = [p.alias for p in all_platforms]
    platforms = [alias_map[a] for a in selected_aliases]

    target_choices = [
        questionary.Choice("Frontend", value="frontend", checked=True),
        questionary.Choice("Server", value="server", checked=True),
    ]
    selected_targets = _ask_or_exit(questionary.checkbox(
        "Targets:", choices=target_choices, style=PROMPT_STYLE,
    ))
    if not selected_targets:
        selected_targets = ["frontend", "server"]
    config = BuildConfig(
        frontend="frontend" in selected_targets,
        server="server" in selected_targets,
    )

    _print_summary(platforms, config)
    return platforms, config


def _print_summary(platforms: list[Platform], config: BuildConfig) -> None:
    targets = [t for t, on in [("frontend", config.frontend), ("server", config.server)] if on]
    plat_str = ", ".join(p.alias for p in platforms)
    console.print(f"  [dim]platforms:[/] {plat_str}")
    console.print(f"  [dim]targets:[/]   {', '.join(targets)}")
    console.print()


def print_results(results: list[PlatformResult], wall_time: float) -> None:
    table = Table(title="Results", title_style="bold")
    table.add_column("Platform", style="cyan", min_width=26)
    table.add_column("Status", justify="center")
    table.add_column("Time", justify="right")
    table.add_column("Artifacts")

    succeeded = 0
    for r in results:
        if r.success:
            status = "[green]OK[/]"
            names = ", ".join(Path(a).name for a in r.artifacts)
            detail = names or "no artifacts"
            succeeded += 1
        else:
            status = "[red]FAIL[/]"
            detail = f"[red]{r.error or 'unknown error'}[/]"
        table.add_row(r.platform.label, status, f"{r.elapsed:.1f}s", detail)

    console.print(table)

    total = len(results)
    color = "green" if succeeded == total else "red"
    console.print(f"\n[{color}]{succeeded}/{total}[/] succeeded in [bold]{wall_time:.1f}s[/] (wall clock)")


def resolve_cli_platforms(raw: str, alias_map: dict[str, Platform]) -> list[Platform]:
    platforms = []
    for alias in raw.split(","):
        alias = alias.strip()
        if alias not in alias_map:
            console.print(f"[red]Unknown platform:[/] {alias}")
            console.print(f"Valid: {', '.join(alias_map.keys())}")
            sys.exit(1)
        platforms.append(alias_map[alias])
    return platforms


def resolve_cli_targets(raw: str) -> BuildConfig:
    cfg = BuildConfig(frontend=False, server=False)
    for t in raw.split(","):
        t = t.strip()
        if t == "frontend":
            cfg.frontend = True
        elif t == "server":
            cfg.server = True
        else:
            console.print(f"[red]Unknown target:[/] {t} (expected: frontend, server)")
            sys.exit(1)
    return cfg


def check_git_clean(root: Path) -> tuple[str, bool]:
    """Return (short SHA, is_clean)."""
    sha = subprocess.check_output(
        ["git", "rev-parse", "--short", "HEAD"], text=True, cwd=root,
    ).strip()
    status = subprocess.check_output(
        ["git", "status", "--porcelain"], text=True, cwd=root,
    ).strip()
    return sha, len(status) == 0


def check_prerequisites(platforms: list[Platform], config: BuildConfig) -> None:
    """Verify required tools are available, fail fast if not."""
    need_xwin = any(p.os == "windows" and p.cross for p in platforms)
    need_cross = any(p.cross and p.os != "windows" for p in platforms)
    need_docker = any(p.cross and p.os == "linux" for p in platforms)

    checks: list[tuple[str, bool]] = [("cargo", True)]
    if need_xwin:
        checks.append(("cargo-xwin", True))
        checks.append(("clang", True))
        checks.append(("cmake", True))
    if need_cross:
        checks.append(("cross", True))
    if need_docker:
        checks.append(("docker", True))

    console.print("[bold]Prerequisites:[/]")
    missing_critical: list[str] = []
    for tool, critical in checks:
        found = shutil.which(tool) is not None
        if not found and critical:
            missing_critical.append(tool)
        if found:
            status = "[green]found[/]"
        elif critical:
            status = "[red]MISSING[/]"
        else:
            status = "[yellow]missing (optional)[/]"
        console.print(f"  {tool}: {status}")

    if missing_critical:
        console.print(f"\n[red]Missing critical tools: {', '.join(missing_critical)}[/]")
        sys.exit(1)
    console.print()


def write_checksums(results: list[PlatformResult], out_dir: Path) -> Path:
    """Write SHA256 checksums for all artifacts."""
    lines: list[str] = []
    for r in results:
        if not r.success:
            continue
        for artifact_path in r.artifacts:
            p = Path(artifact_path)
            if p.is_dir():
                continue
            h = hashlib.sha256(p.read_bytes()).hexdigest()
            lines.append(f"SHA256 ({p.name}) = {h}")
    lines.sort()
    checksum_file = out_dir / "checksums.sha256"
    checksum_file.write_text("\n".join(lines) + "\n")
    return checksum_file


def main() -> None:
    parser = argparse.ArgumentParser(description="Sova release builder")
    parser.add_argument("--platforms", help="Comma-separated: macos-arm64,macos-x86_64,linux-x86_64,linux-aarch64,windows-x86_64")
    parser.add_argument("--targets", help="Comma-separated: frontend,server")
    parser.add_argument("--all", action="store_true", help="Build all platforms and targets")
    parser.add_argument("--verbose", "-v", action="store_true", help="Show build logs for all platforms (not just failures)")
    parser.add_argument("--force", action="store_true", help="Allow building from a dirty git tree")
    parser.add_argument("--no-checksums", action="store_true", help="Skip SHA256 checksum generation")
    args = parser.parse_args()

    root = Path(subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip())

    all_platforms = load_platforms(root)
    alias_map = {p.alias: p for p in all_platforms}

    version = get_version(root)
    sha, clean = check_git_clean(root)
    dirty_tag = "" if clean else ", dirty"
    console.print(Panel(f"Sova [bold]{version}[/] ({sha}{dirty_tag}) — release builder", style="blue"))

    if not clean and not args.force:
        console.print("[red]Working tree is dirty. Commit your changes or use --force.[/]")
        sys.exit(1)

    if args.all:
        platforms = list(all_platforms)
        config = BuildConfig()
    elif args.platforms or args.targets:
        platforms = resolve_cli_platforms(args.platforms, alias_map) if args.platforms else list(all_platforms)
        config = resolve_cli_targets(args.targets) if args.targets else BuildConfig()
    else:
        platforms, config = prompt_interactive(all_platforms, alias_map)

    check_prerequisites(platforms, config)

    _prebuild_cross_images(root, platforms)

    t0 = time.monotonic()
    results = run_builds(root, platforms, config, version, verbose=args.verbose)
    wall_time = time.monotonic() - t0

    print_results(results, wall_time)

    if not args.no_checksums and any(r.success for r in results):
        checksum_file = write_checksums(results, root / OUT)
        console.print(f"[green]Checksums written to {checksum_file}[/]")

    if any(not r.success for r in results):
        sys.exit(1)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        console.print("\n[yellow]Interrupted.[/] Partial artifacts may remain in releases/.")
        sys.exit(130)
