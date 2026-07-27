---
title: Installation
description: Install Bitbucket CLI with Cargo, a prebuilt binary, or a platform package.
---

The binary is named `bitbucket`. After installing, verify it with:

```bash
bitbucket --version
```

## Using Cargo (recommended)

If you have the [Rust toolchain](https://rustup.rs/) installed:

```bash
cargo install bitbucket-cli
```

## Debian / Ubuntu

```bash
wget https://github.com/quinnjr/bitbucket-cli/releases/latest/download/bitbucket-cli_amd64.deb
sudo dpkg -i bitbucket-cli_amd64.deb
```

## Red Hat / Fedora / CentOS

```bash
wget https://github.com/quinnjr/bitbucket-cli/releases/latest/download/bitbucket-cli.x86_64.rpm
sudo rpm -i bitbucket-cli.x86_64.rpm
# or with dnf
sudo dnf install bitbucket-cli.x86_64.rpm
```

## Arch Linux

The recommended path builds a real package from the released `PKGBUILD`:

```bash
wget https://github.com/quinnjr/bitbucket-cli/releases/latest/download/PKGBUILD
makepkg -si
```

Or download the plain binary tarball and install it manually:

```bash
wget https://github.com/quinnjr/bitbucket-cli/releases/latest/download/bitbucket-cli-vX.X.X-x86_64-linux.tar.zst
tar --zstd -xf bitbucket-cli-vX.X.X-x86_64-linux.tar.zst
sudo mv bitbucket /usr/local/bin/
```

## Alpine Linux

```bash
wget https://github.com/quinnjr/bitbucket-cli/releases/latest/download/bitbucket-cli-vX.X.X-alpine-x86_64.tar.gz
tar -xzf bitbucket-cli-vX.X.X-alpine-x86_64.tar.gz
sudo mv bitbucket /usr/local/bin/
```

## Windows

Download the MSI installer from the [Releases](https://github.com/quinnjr/bitbucket-cli/releases) page and run it.

## From source

```bash
git clone https://github.com/quinnjr/bitbucket-cli.git
cd bitbucket-cli
cargo install --path .
```

## Prebuilt binaries

Prebuilt binaries for Linux (x86_64 / arm64), macOS (Intel / Apple Silicon), and Windows are attached to every [GitHub release](https://github.com/quinnjr/bitbucket-cli/releases).
