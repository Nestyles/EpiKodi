# Project Overview

## What is EpiKodi?

EpiKodi is a modern desktop media player and management system designed as an alternative to Kodi. Built with cutting-edge web technologies and packaged as a native desktop application, EpiKodi provides a smooth, responsive user experience for managing and playing your media collection.

## Why EpiKodi?

### Advantages

- **Modern Stack**: Built with React and Rust for optimal performance and maintainability
- **Cross-Platform**: Single codebase runs natively on Windows and macOS
- **lightweight**: Smaller footprint compared to traditional media center solutions
- **Active Development**: Regular updates and community contributions
- **Open Source**: MIT licensed with transparent development

## Architecture Overview

EpiKodi follows a client-server architecture:

```
┌─────────────────────────────────────┐
│     React Frontend (TypeScript)      │
│  ┌────────────────────────────────┐  │
│  │ Components & UI                │  │
│  │ - VideoPlayer                  │  │
│  │ - Library Browser              │  │
│  │ - Settings Panel               │  │
│  └────────────────────────────────┘  │
│                                       │
│  Tauri IPC Bridge                    │
└─────────────────────────────────────┘
          ↕
┌─────────────────────────────────────┐
│     Rust Backend (Tauri)            │
│  ┌────────────────────────────────┐  │
│  │ Command Handlers               │  │
│  │ File System Operations         │  │
│  │ Database Queries               │  │
│  │ Plugin Integration             │  │
│  └────────────────────────────────┘  │
└─────────────────────────────────────┘
```

## Technology Highlights

### Frontend Stack
- **React 19**: Latest UI library for component-based development
- **TypeScript**: Type-safe JavaScript development
- **Chakra UI**: Accessible, customizable component library
- **Vite**: Ultra-fast build tool and development server

### Backend Stack
- **Rust**: For performance-critical operations
- **Tauri 2.x**: Minimal overhead desktop framework
- **SQLite**: Lightweight embedded database

### Communication
- **Tauri IPC**: Fast inter-process communication between frontend and backend
- Command-based architecture for clean separation of concerns

## Project Goals

1. **Simplify Media Management**: Provide an intuitive interface for media discovery and playback
2. **Maintain Performance**: Deliver responsive, lag-free user experience
3. **Foster Community**: Enable community contributions and extensions
4. **Cross-Platform Excellence**: Ensure consistent experience across all supported platforms
5. **Modern Development**: Leverage current best practices in web and systems programming

## Development Workflow

The project uses a monorepo structure with:

- **PNPM**: Fast, disk-space-efficient package manager
- **Vite**: Instant hot module replacement during development
- **TypeScript**: Throughout the stack for consistency
- **Cargo**: Rust package management for backend code

This setup enables rapid development cycles with instant feedback and type safety across the entire application.
