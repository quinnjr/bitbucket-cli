# Packaging Setup Summary

This document summarizes the comprehensive packaging support added to Bitbucket CLI.

## What Was Added

### 1. Package Format Support

The following package formats are now supported:

- ✅ **Debian/Ubuntu** (`.deb`)
- ✅ **Red Hat/Fedora/CentOS** (`.rpm`)
- ✅ **Arch Linux** (PKGBUILD built with `makepkg`, plus a generic binary tarball `.tar.zst`)
- ✅ **Alpine Linux** (static musl `.tar.gz`)
- ✅ **Windows** (`.msi` installer)

### 2. Configuration Files

#### Cargo.toml
- Added `[package.metadata.deb]` section for Debian package configuration
- Added `[package.metadata.generate-rpm]` section for RPM package configuration

#### Packaging Directory (`packaging/`)
- `packaging/arch/PKGBUILD` - Arch Linux package build script
- `packaging/alpine/APKBUILD` - Alpine Linux package build script
- `packaging/windows/main.wxs` - WiX configuration for Windows MSI installer
- `packaging/README.md` - Overview of all packaging options

### 3. Build Scripts

#### Linux/macOS
`scripts/build-packages.sh` - Automated build script supporting:
- Build individual packages: `./scripts/build-packages.sh deb`
- Build all packages: `./scripts/build-packages.sh all`
- Supported targets: deb, rpm, arch, alpine

#### Windows
`scripts/build-packages.ps1` - PowerShell script for building MSI installers

### 4. CI/CD Integration

Updated `.github/workflows/release.yml` with new `build-packages` job that:
- Builds all package types automatically on release tags
- Uploads packages to GitHub Releases
- Runs in parallel with existing binary builds

### 5. Documentation

- `docs/PACKAGING.md` - Comprehensive guide for building packages
- `packaging/windows/README.md` - Windows-specific packaging documentation
- Updated `README.md` with installation instructions for all package types
- Updated `CONTRIBUTING.md` with packaging development guidelines

### 6. .gitignore Updates

Added exclusions for packaging artifacts:
- `*.deb`, `*.rpm`, `*.pkg.tar.zst`, `*.msi`, `*.apk`
- Build directories for Arch and Alpine packages

## Package Details

### Debian/Ubuntu (.deb)
- Uses `cargo-deb`
- Automatically determines dependencies
- Installs to `/usr/bin/bitbucket`
- Includes documentation in `/usr/share/doc/bitbucket-cli/`

### Red Hat/Fedora/CentOS (.rpm)
- Uses `cargo-generate-rpm`
- Requires `dbus-libs`
- Installs to `/usr/bin/bitbucket`
- Includes documentation in `/usr/share/doc/bitbucket-cli/`

### Arch Linux
- Provides PKGBUILD (release asset and `packaging/arch/PKGBUILD`) for building a real pacman package with `makepkg -si`, ready for AUR submission
- Releases also ship `bitbucket-cli-v<version>-x86_64-linux.tar.zst`, a plain zstd tarball of the `bitbucket` binary (not a pacman package — `pacman -U` does not work on it)
- Follows Arch packaging standards
- Ready for community distribution

### Alpine Linux
- Statically-linked musl binary
- No external dependencies
- Distributed as `.tar.gz`
- Ideal for containers and minimal systems

### Windows (.msi)
- Uses WiX Toolset
- Automatic PATH configuration
- System-wide or per-user installation
- Includes uninstaller

## How to Use

### For End Users

Install from releases:
```bash
# Debian/Ubuntu
wget https://github.com/pegasusheavy/bitbucket-cli/releases/latest/download/bitbucket-cli_amd64.deb
sudo dpkg -i bitbucket-cli_amd64.deb

# Red Hat/Fedora
wget https://github.com/pegasusheavy/bitbucket-cli/releases/latest/download/bitbucket-cli.x86_64.rpm
sudo rpm -i bitbucket-cli.x86_64.rpm

# Arch Linux (recommended: download the PKGBUILD and build the package)
wget https://github.com/pegasusheavy/bitbucket-cli/releases/latest/download/PKGBUILD
makepkg -si

# Arch Linux alternative: download the binary tarball and extract it manually
wget https://github.com/pegasusheavy/bitbucket-cli/releases/latest/download/bitbucket-cli-vX.X.X-x86_64-linux.tar.zst
tar --zstd -xf bitbucket-cli-vX.X.X-x86_64-linux.tar.zst
sudo mv bitbucket /usr/local/bin/

# Alpine Linux
wget https://github.com/pegasusheavy/bitbucket-cli/releases/latest/download/bitbucket-cli-vX.X.X-alpine-x86_64.tar.gz
tar -xzf bitbucket-cli-vX.X.X-alpine-x86_64.tar.gz
sudo mv bitbucket /usr/local/bin/

# Windows - download and run the .msi installer
```

### For Developers

Build packages locally:
```bash
# Linux/macOS
./scripts/build-packages.sh all

# Windows
.\scripts\build-packages.ps1
```

### For Release Process

Automated releases work by tagging:
```bash
git tag v0.3.0
git push origin v0.3.0
```

GitHub Actions will automatically:
1. Build all package types
2. Upload to GitHub Releases
3. Publish to crates.io

## Files Modified

1. `Cargo.toml` - Added packaging metadata
2. `.gitignore` - Added packaging artifact exclusions
3. `.github/workflows/release.yml` - Added packaging job
4. `README.md` - Updated installation section
5. `CONTRIBUTING.md` - Added packaging development section

## Files Created

1. `packaging/arch/PKGBUILD`
2. `packaging/alpine/APKBUILD`
3. `packaging/windows/main.wxs`
4. `packaging/windows/README.md`
5. `packaging/README.md`
6. `scripts/build-packages.sh`
7. `scripts/build-packages.ps1`
8. `docs/PACKAGING.md`

## Next Steps

To complete the packaging setup:

1. **Windows Icon**: Replace `packaging/windows/icon.ico` with an actual application icon
2. **Test Locally**: Run build scripts to verify packages build correctly
3. **AUR Submission**: Submit PKGBUILD to Arch User Repository
4. **Alpine APK**: Submit APKBUILD to Alpine package repository
5. **Distribution Testing**: Test installation on each supported platform

## Dependencies

### Build Tools Required

- **Debian/Ubuntu**: `cargo-deb`, `libdbus-1-dev`
- **Red Hat/Fedora**: `cargo-generate-rpm`, `dbus-devel`
- **Arch Linux**: `zstd`
- **Alpine Linux**: `musl-tools`
- **Windows**: WiX Toolset 4.0+

## Future Enhancements

Potential additions:
- Homebrew formula for macOS
- Chocolatey package for Windows
- Snap package for Ubuntu
- Flatpak for universal Linux distribution
- Docker images

## Support

For packaging-related issues:
- See detailed guide: `docs/PACKAGING.md`
- Check CI logs in GitHub Actions
- Open an issue with `packaging` label
