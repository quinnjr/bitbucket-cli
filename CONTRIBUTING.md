# Contributing to Bitbucket CLI

First off, thank you for considering contributing to Bitbucket CLI! It's people like you that make this project great.

## Code of Conduct

This project and everyone participating in it is governed by our commitment to creating a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When you create a bug report, include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples**
- **Describe the behavior you observed and what you expected**
- **Include your environment details** (OS, Rust version, etc.)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion:

- **Use a clear and descriptive title**
- **Provide a detailed description of the proposed enhancement**
- **Explain why this enhancement would be useful**
- **List any alternatives you've considered**

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Install dependencies**: Make sure you have Rust installed via [rustup](https://rustup.rs/)
3. **Make your changes**: Follow the coding standards below
4. **Test your changes**: Run `cargo test` and `cargo clippy`
5. **Format your code**: Run `cargo fmt`
6. **Commit your changes**: Use clear, descriptive commit messages
7. **Push and create a PR**: Fill out the PR template completely

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/bitbucket-cli.git
cd bitbucket-cli

# Build the project
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy --all-features

# Format code
cargo fmt
```

## Coding Standards

- Follow Rust's official style guidelines
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Write tests for new functionality
- Document public APIs with doc comments
- Keep functions focused and small
- Use meaningful variable and function names

### Command taxonomy

The top-level commands (`auth`, `repo`, `pr`, `issue`, `pipeline`, `workspace`,
`user`) mirror the major Bitbucket resource types. A resource that is *owned by
a repository* is a subcommand group under `repo` (e.g. `repo branch`,
`repo webhook`, `repo permission`) rather than a new top-level command. `pr`,
`issue`, and `pipeline` are top-level for historical reasons and because they
are the day-to-day nouns; do not add new top-level commands for repo-scoped
resources. When a resource has more than one verb (list/create/delete), group
them under a subcommand (`repo download {list,get,delete}`), never as flat
sibling verbs (`download-list`, `download-get`).

### Pagination and limits

All paginated list commands share one policy via `cli::pagination`: a
`--limit` (default 25) capped at 100 with a notice, using `effective_limit`.
Do not add a per-command `MAX_LIMIT`, a clap `range(1..=100)` validator, or a
bespoke cap — reuse the shared helper so behavior and wording stay uniform.

## Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests when relevant

### Commit Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Example: `feat: add pipeline trigger command with --wait flag`

## Project Structure

```
src/
├── main.rs          # Entry point
├── cli/             # CLI command definitions
├── api/             # Bitbucket API client
├── auth/            # Authentication handling
├── config/          # Configuration management
├── models/          # Data structures
└── tui/             # Terminal UI
packaging/           # Distribution package configs
├── arch/            # Arch Linux PKGBUILD
├── alpine/          # Alpine Linux APKBUILD
└── windows/         # Windows MSI configuration
scripts/             # Build and utility scripts
docs/                # Documentation site
```

## Versioning

`Cargo.toml` is the single source of truth for the version. Several other files hardcode it (the Windows MSI config, the Arch and Alpine package files, and two docs pages) and must never be edited by hand.

To bump the version:

```bash
# Edit the version in Cargo.toml, then:
./scripts/bump-version.sh

# Or do both in one step:
./scripts/bump-version.sh 0.5.0
```

The script propagates the Cargo.toml version to every derived location and fails loudly if any expected version string has moved or gone missing.

## Building Packages

If you're working on packaging or release automation, see [docs/PACKAGING.md](docs/PACKAGING.md) for detailed information on building distribution packages for various platforms.

Quick reference:
```bash
# Linux/macOS - Build all packages
./scripts/build-packages.sh all

# Windows - Build MSI installer
.\scripts\build-packages.ps1
```

## Questions?

Feel free to open a discussion or issue if you have any questions!
