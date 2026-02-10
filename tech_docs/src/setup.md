# Development Setup Guide

## Introduction

This guide will walk you through setting up the EpiKodi development environment. Whether you're looking to contribute, modify the application, or simply explore the codebase, this setup ensures you have everything needed to run and develop EpiKodi locally.

### What You'll Learn

This guide covers:
- System requirements and dependencies
- Installation and configuration steps
- Running the development server
- Building production binaries
- Troubleshooting common issues

### Prerequisites

Before you begin, ensure you have a working knowledge of:
- Command line/terminal usage
- Git version control
- Basic Node.js and Rust concepts

## System Requirements

### Required Tools

Ensure you have the following installed before proceeding:

- **Node.js** 20.0.0 or higher
  - [Download from nodejs.org](https://nodejs.org)
  - Verify installation: `node --version`

- **Rust** 1.88.0 or higher
  - [Install from rustup.rs](https://rustup.rs)
  - Verify installation: `rustc --version` and `cargo --version`

- **pnpm** 8.10.5 or higher (Node package manager)
  - [Installation guide](https://pnpm.io/installation)
  - Verify installation: `pnpm --version`
  - Alternative: Use `npm` (8.10.5+) or `yarn` (1.22.0+)

### Optional Tools

For enhanced development experience, consider installing:

- **Git** for version control
- **Visual Studio Code** with recommended extensions
- **Rust Analyzer** VS Code extension for IDE support

## Installation Steps

### 1. Clone the Repository

```bash
git clone https://github.com/Nestyles/EpiKodi.git
cd EpiKodi
```

### 2. Install Dependencies

Navigate to the project root and install all dependencies:

```bash
pnpm install
```

This command will:
- Download and install all Node.js packages
- Set up the Tauri development environment
- Configure the Rust build environment

### 3. Verify Installation

Test that everything is set up correctly:

```bash
node --version    # Should be 20.0.0 or higher
rustc --version   # Should be 1.88.0 or higher
pnpm --version    # Should be 8.10.5 or higher
cargo --version   # Rust package manager
```

## Running the Development Server

### Starting Development Mode

To launch the application in development mode with hot-reload:

```bash
pnpm tauri dev
```

This command:
- Compiles the Rust backend
- Starts the Vite development server
- Launches the Tauri window with your application
- Enables hot module replacement (HMR) for React components

### Frontend-Only Development

If you want to work on the frontend without the Tauri wrapper:

```bash
pnpm dev
```

This starts a web-only development server at `http://localhost:5173`.

## Building for Production

### Creating Distribution Binaries

To build production-ready executables:

```bash
pnpm build
```

This command:
- Compiles React with TypeScript optimizations
- Bundles assets with Vite
- Produces a Rust binary with all assets embedded
- Creates installer executables for your platform

### Build Artifacts

After building, you'll find:
- **Windows**: `.msi` installer and `.exe` in `src-tauri/target/release/bundle/msi/`
- **macOS**: `.dmg` installer and `.app` bundle in `src-tauri/target/release/bundle/macos/`

## Data Storage

### Application Data Location

EpiKodi stores user data and configuration in platform-specific directories:

- **Windows**: `%appdata%\epikodi\`
- **macOS**: `~/Library/Application Support/epikodi/`
- **Linux**: `~/.config/epikodi/`

### What's Stored

- User preferences and settings
- Library metadata
- Application logs
- Cached content information

You can manually access these directories to debug issues or manage data.

## Project Structure Overview

```
epikodi/
├── src/                    # Frontend code (React + TypeScript)
│   ├── components/        # React components
│   ├── lib/              # Utilities and helpers
│   └── assets/           # Static resources
├── src-tauri/            # Backend code (Rust)
│   ├── src/              # Rust source files
│   └── Cargo.toml        # Rust dependencies
├── tech_docs/            # Technical documentation
├── package.json          # Node.js dependencies
├── pnpm-lock.yaml        # Locked dependency versions
├── tsconfig.json         # TypeScript configuration
└── vite.config.ts        # Vite build configuration
```

## Troubleshooting

### Port Already in Use

If port 5173 is occupied:

```bash
# Kill the process on Windows
netstat -ano | findstr :5173
taskkill /PID <PID> /F

# Or on macOS/Linux
lsof -ti:5173 | xargs kill -9
```

### Rust Build Errors

If you encounter Rust compilation errors:

```bash
# Update Rust to the latest version
rustup update

# Clean build artifacts
cargo clean

# Then try again
pnpm tauri dev
```

### Node Dependencies Issues

If experiencing package-related problems:

```bash
# Clear pnpm cache
pnpm store prune

# Reinstall dependencies
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

### Tauri Development Issues

For Tauri-specific problems:

- Check [Tauri documentation](https://tauri.app/docs)
- Review platform-specific setup requirements
- Check GitHub Issues for known problems

## Next Steps

Now that you have EpiKodi running, you can:

- Explore the codebase in [Project Overview](./chapter_1.md)
- Read about the [Architecture](./architecture.md) (coming soon)
- Check the [Contributing Guide](../CONTRIBUTING.md) for contribution guidelines
- Start working on features or bug fixes

## Getting Help

If you encounter issues during setup:

1. Check this guide's troubleshooting section
2. Review existing [GitHub Issues](https://github.com/Nestyles/EpiKodi/issues)
3. Read the [Tauri documentation](https://tauri.app/docs)
4. Open a new issue with detailed error messages and system information

Happy coding!
