#!/usr/bin/env python3
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

# Core OS, driver, and graphics libraries that MUST come from the host OS
SYSTEM_LIB_BLACKLIST = [
    "linux-vdso",
    "libc\\.so",
    "libm\\.so",
    "libdl\\.so",
    "libpthread\\.so",
    "librt\\.so",
    "ld-linux",
    "libGL",
    "libEGL",
    "libdrm",
    "libxcb",
    "libX11",
    "libwayland",
    "libudev",
]

def patch_appimage(appimage_path: Path):
    """Patches the magic bytes in the generated AppImage header."""
    if not appimage_path.exists():
        return

    print(f"==> Patching magic bytes in {appimage_path.name}...")
    with open(appimage_path, "rb+") as f:
        content = f.read()
        target = b"AI\x02"
        replacement = b"\x00\x00\x00"

        if target in content:
            patched_content = content.replace(target, replacement, 1)  # Patch first match in header
            f.seek(0)
            f.write(patched_content)
            f.truncate()
            print("  [Patch] Replaced magic bytes 'AI\\x02' -> '\\x00\\x00\\x00'")
        else:
            print("  [Patch] Magic bytes 'AI\\x02' not found (already patched or different header).")

def get_cargo_metadata() -> dict:
    """Runs cargo metadata to get target directory and package information."""
    try:
        res = subprocess.run(
            ["cargo", "metadata", "--format-version=1"],
            capture_output=True,
            text=True,
            check=True,
        )
        return json.loads(res.stdout)
    except (subprocess.CalledProcessError, FileNotFoundError) as e:
        print(f"Error executing 'cargo metadata': {e}", file=sys.stderr)
        sys.exit(1)


def find_binary_path(metadata: dict, bin_name: str, target: str | None, profile: str) -> Path:
    """Locates the binary in Cargo's build directory."""
    target_dir = Path(metadata["target_directory"])
    
    if target:
        bin_dir = target_dir / target / profile
    else:
        bin_dir = target_dir / profile

    bin_path = bin_dir / bin_name
    if not bin_path.exists():
        print(f"Error: Binary not found at '{bin_path}'. Did you run 'cargo build'?", file=sys.stderr)
        sys.exit(1)

    return bin_path


def resolve_ldd_deps(bin_path: Path) -> list[Path]:
    """Runs ldd on the binary and returns non-blacklisted dynamic library paths."""
    res = subprocess.run(["ldd", str(bin_path)], capture_output=True, text=True, check=True)
    blacklist_regex = re.compile("|".join(SYSTEM_LIB_BLACKLIST))
    libs = []

    for line in res.stdout.splitlines():
        match = re.search(r"=>\s+([^\s]+)", line)
        if match:
            lib_path = Path(match.group(1))
            if lib_path.exists() and not blacklist_regex.search(lib_path.name):
                print(f"  [Bundle] {lib_path.name} -> {lib_path}")
                libs.append(lib_path)

    return libs


def build_appimage(bin_name: str, bin_path: Path, output_path: Path, icon_path: Path | None):
    appdir = Path(f"target/AppDir_{bin_name}")
    if appdir.exists():
        shutil.rmtree(appdir)

    # 1. Directory Structure
    (appdir / "usr/bin").mkdir(parents=True, exist_ok=True)
    (appdir / "usr/lib").mkdir(parents=True, exist_ok=True)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # 2. Copy Binary
    print(f"==> Copying binary {bin_path}...")
    shutil.copy2(bin_path, appdir / f"usr/bin/{bin_name}")

    # 3. Resolve & Copy .so Dependencies
    print("==> Resolving dependencies with ldd...")
    deps = resolve_ldd_deps(bin_path)
    for dep in deps:
        shutil.copy2(dep.resolve(), appdir / "usr/lib" / dep.name)

    # 4. AppRun Entrypoint Script
    print("==> Generating AppRun...")
    apprun_content = f"""#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
export LD_LIBRARY_PATH="${{HERE}}/usr/lib:${{LD_LIBRARY_PATH}}"
exec "${{HERE}}/usr/bin/{bin_name}" "$@"
"""
    apprun = appdir / "AppRun"
    apprun.write_text(apprun_content)
    apprun.chmod(0o755)

    # 5. Desktop File
    print("==> Generating Desktop Entry...")
    desktop_content = f"""[Desktop Entry]
Name={bin_name}
Exec={bin_name}
Icon={bin_name}
Type=Application
Categories=Utility;
"""
    (appdir / f"{bin_name}.desktop").write_text(desktop_content)

    # 6. Icon Handling
    icon_dest = appdir / f"{bin_name}.png"
    if icon_path and icon_path.exists():
        shutil.copy2(icon_path, icon_dest)
    else:
        icon_dest.touch()

    # 8. Package AppImage
    print(f"==> Packaging AppImage to {output_path}...")
    env = os.environ.copy()
    env["ARCH"] = "x86_64"
    env["APPIMAGE_EXTRACT_AND_RUN"] = "1"

    subprocess.run(["appimagetool", str(appdir), str(output_path)], stdout=subprocess.DEVNULL, env=env, check=True)
    print(f"\nBuilt AppImage: \"{output_path}\"\n")

    # 9. Apply binary patch to the generated AppImage
    patch_appimage(output_path)

    print(f"\n✨ Done! Built AppImage: {output_path}")

def main():
    parser = argparse.ArgumentParser(description="Create an AppImage from Cargo metadata.")
    parser.add_argument("--bin", required=True, help="Binary name to package")
    parser.add_argument("--output", required=True, help="Destination AppImage path")
    parser.add_argument("--target", help="Cargo target triple (e.g. x86_64-unknown-linux-gnu)")
    parser.add_argument("--profile", default="release", help="Build profile (default: release)")
    parser.add_argument("--icon", help="Path to PNG icon file")

    args = parser.parse_args()

    metadata = get_cargo_metadata()
    bin_path = find_binary_path(metadata, args.bin, args.target, args.profile)
    output_path = Path(args.output).resolve()
    icon_path = Path(args.icon) if args.icon else None

    build_appimage(args.bin, bin_path, output_path, icon_path)


if __name__ == "__main__":
    main()