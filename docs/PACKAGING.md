# Packaging Guide

This guide explains how to build distribution packages for Bitbucket CLI.

## Overview

Bitbucket CLI supports the following package formats:

- **Debian/Ubuntu**: `.deb` packages
- **Red Hat/Fedora/CentOS**: `.rpm` packages
- **Arch Linux**: PKGBUILD built with `makepkg`; releases also ship a generic `bitbucket-cli-v<version>-x86_64-linux.tar.zst` binary tarball
- **Alpine Linux**: Static musl binaries in `.tar.gz`
- **Windows**: `.msi` installers

## Prerequisites

### All Platforms
- Rust toolchain (1.85+)
- Git

### Linux
- `libdbus-1-dev` (Debian/Ubuntu) or `dbus-devel` (Red Hat/Fedora)
- `zstd` (for Arch packages)
- `musl-tools` (for Alpine packages)

### Windows
- WiX Toolset 4.0+
- Visual Studio Build Tools

## Building Packages

### Automated Build Script

#### Linux/macOS

Use the provided shell script to build packages:

```bash
# Build all packages
./scripts/build-packages.sh all

# Build specific package
./scripts/build-packages.sh deb
./scripts/build-packages.sh rpm
./scripts/build-packages.sh arch
./scripts/build-packages.sh alpine
```

#### Windows

Use the PowerShell script:

```powershell
.\scripts\build-packages.ps1
```

### Manual Building

#### Debian Package

```bash
# Install cargo-deb
cargo install cargo-deb

# Build package
cargo deb

# Output: target/debian/bitbucket-cli_*.deb
```

Configuration is in `Cargo.toml` under `[package.metadata.deb]`.

#### RPM Package

```bash
# Install cargo-generate-rpm
cargo install cargo-generate-rpm

# Build binary first
cargo build --release

# Generate RPM
cargo generate-rpm

# Output: target/generate-rpm/bitbucket-cli-*.rpm
```

Configuration is in `Cargo.toml` under `[package.metadata.generate-rpm]`.

#### Arch Linux Package

The `arch` target of `scripts/build-packages.sh` produces a generic Linux binary
tarball named `bitbucket-cli-<version>-x86_64-linux.tar.zst` (not a pacman
package):

```bash
# Build binary
cargo build --release

# Create compressed archive (what scripts/build-packages.sh arch does)
cd target/release
tar -I 'zstd -19' -cf ../bitbucket-cli-<version>-x86_64-linux.tar.zst bitbucket
```

A real Arch package is built from the PKGBUILD:

```bash
cd packaging/arch
makepkg -si
```

#### Alpine Linux

```bash
# Add musl target
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl

# Create tarball
cd target/x86_64-unknown-linux-musl/release
tar -czvf bitbucket-cli-alpine-x86_64.tar.gz bitbucket
```

#### Windows MSI

```bash
# Install WiX
dotnet tool install --global wix
wix extension add -g WixToolset.UI.wixext

# Build binary
cargo build --release --target x86_64-pc-windows-msvc

# Build installer
wix build -arch x64 -o bitbucket-cli.msi packaging/windows/main.wxs
```

The MSI references `packaging/windows/icon.ico` as the Add/Remove Programs
product icon. The committed file is generated deterministically by
`scripts/generate-icon.py`; run it with `--check` to verify the bytes, or
without arguments to regenerate after changing the design.

## Package Contents

All packages include:

- `bitbucket` binary (or `bitbucket.exe` on Windows)
- README.md
- LICENSE file

### Installation Locations

- **Linux binaries**: `/usr/bin/bitbucket`
- **Linux docs**: `/usr/share/doc/bitbucket-cli/`
- **Windows binary**: `C:\Program Files\BitbucketCLI\bitbucket.exe`
- **Windows docs**: `C:\Program Files\BitbucketCLI\`

## Automated Releases

Packages are automatically built and published via GitHub Actions when a new version is tagged:

```bash
git tag v0.3.0
git push origin v0.3.0
```

The workflow will:
1. Build binaries for all platforms
2. Create distribution packages
3. Upload everything to GitHub Releases
4. Publish to crates.io

## Distribution-Specific Notes

### Debian/Ubuntu

Dependencies are automatically determined by cargo-deb. The package requires:
- `libdbus-1-3`

Install with:
```bash
sudo dpkg -i bitbucket-cli_*.deb
# or
sudo apt install ./bitbucket-cli_*.deb
```

### Red Hat/Fedora/CentOS

The RPM package requires:
- `dbus-libs`

Install with:
```bash
sudo rpm -i bitbucket-cli-*.rpm
# or
sudo dnf install bitbucket-cli-*.rpm
# or
sudo yum install bitbucket-cli-*.rpm
```

### Arch Linux

The PKGBUILD can be submitted to AUR (Arch User Repository) for community distribution.

Dependencies:
- `dbus`
- `gcc-libs`

Install with:
```bash
# Recommended: download the PKGBUILD from releases and build the package
wget https://github.com/quinnjr/bitbucket-cli/releases/latest/download/PKGBUILD
makepkg -si

# Or download the binary tarball and extract it manually
# (it is a plain zstd tarball, not a pacman package — pacman -U does not work on it)
wget https://github.com/quinnjr/bitbucket-cli/releases/latest/download/bitbucket-cli-vX.X.X-x86_64-linux.tar.zst
tar --zstd -xf bitbucket-cli-vX.X.X-x86_64-linux.tar.zst
sudo mv bitbucket /usr/local/bin/
```

### Alpine Linux

The Alpine package is a statically-linked musl binary, requiring no external dependencies.

Install with:
```bash
tar -xzf bitbucket-cli-alpine-*.tar.gz
sudo mv bitbucket /usr/local/bin/
sudo chmod +x /usr/local/bin/bitbucket
```

### Windows

The MSI installer:
- Adds `bitbucket.exe` to PATH automatically
- Can be installed system-wide or per-user
- Includes uninstaller

Install by double-clicking the `.msi` file or via command line:
```powershell
msiexec /i bitbucket-cli.msi
```

## Testing Packages

### Debian/Ubuntu
```bash
# Install
sudo dpkg -i bitbucket-cli_*.deb

# Test
bitbucket --version

# Uninstall
sudo apt remove bitbucket-cli
```

### Red Hat/Fedora
```bash
# Install
sudo rpm -i bitbucket-cli-*.rpm

# Test
bitbucket --version

# Uninstall
sudo rpm -e bitbucket-cli
```

### Arch Linux
```bash
# Install (build the package from the PKGBUILD)
cd packaging/arch
makepkg -si

# Test
bitbucket --version

# Uninstall
sudo pacman -R bitbucket-cli
```

### Windows
```powershell
# Install
msiexec /i bitbucket-cli.msi /qn

# Test
bitbucket --version

# Uninstall
msiexec /x bitbucket-cli.msi /qn
```

## Troubleshooting

### cargo-deb fails
Ensure `libdbus-1-dev` is installed:
```bash
sudo apt-get install libdbus-1-dev
```

### cargo-generate-rpm fails
Build the release binary first:
```bash
cargo build --release
```

### musl build fails on Linux
Install musl tools:
```bash
# Debian/Ubuntu
sudo apt-get install musl-tools

# Fedora
sudo dnf install musl-gcc
```

### WiX build fails on Windows
Ensure WiX Toolset is installed and in PATH:
```powershell
dotnet tool list --global
# Should show 'wix'
```

## Contributing

When adding new features that affect packaging:

1. Update package metadata in `Cargo.toml`
2. Update packaging configs in `packaging/` directory
3. Test package builds locally
4. Update this documentation
5. Ensure CI builds succeed

## Resources

- [cargo-deb documentation](https://github.com/kornelski/cargo-deb)
- [cargo-generate-rpm documentation](https://github.com/cat-in-136/cargo-generate-rpm)
- [Arch PKGBUILD guide](https://wiki.archlinux.org/title/PKGBUILD)
- [Alpine APKBUILD guide](https://wiki.alpinelinux.org/wiki/Creating_an_Alpine_package)
- [WiX Toolset documentation](https://wixtoolset.org/docs/)
