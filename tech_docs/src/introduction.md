# EpiKodi Technical Documentation

## Overview

EpiKodi is a modern, open-source media station application built as a powerful Kodi alternative. It enables users to view details, play, and share content automatically through an intuitive and responsive desktop interface. This documentation covers the technical architecture, setup, and development guidelines for the EpiKodi project.

## Platform & Architecture

- **Desktop Application**: Tauri-based (Rust backend) with React frontend
- **Monorepo Structure**: Multi-language ecosystem combining Rust, TypeScript, and JavaScript
- **Cross-Platform**: Available for macOS and Windows with native performance

## Technology Stack

### Core Technologies

- **Frontend**: React 19+ with TypeScript
- **Backend**: Rust (via Tauri 2.x)
- **Desktop Framework**: Tauri 2.x
- **UI Component Library**: Chakra UI v3.30
- **Build Tools**: Vite 7.x, TypeScript 5.8
- **Database**: SQLite
- **Styling**: Emotion CSS-in-JS

### Key Dependencies

- **Media Playback**: React Player, Media Chrome
- **Navigation**: React Router DOM v7
- **Theming**: Next Themes for dark/light mode support
- **Dialogs & File Operations**: Tauri Plugin Dialog, Tauri Plugin Opener
- **Logging**: Tauri Plugin Log for application logging
- **Icons**: React Icons for UI iconography
- **API Communication**: Tauri App API v2 for IPC

## Project Structure

### Frontend (`src/`)

- **Components**: Reusable React components including:
  - `VideoPlayer`: Media playback component
  - `Library`: Media library display
  - `Details`: Content details viewer
  - `Settings`: Application settings
  - `Navbar`: Navigation header
- **UI Utilities**: Custom UI helpers and providers:
  - `color-mode`: Theme switching
  - `provider`: Context providers
  - `toaster`: Notification system
  - `tooltip`: Tooltip components
- **Tauri Commands**: Interface layer for backend communication (`tauri-commands.ts`)
- **Assets**: Static resources and media files

### Backend (`src-tauri/`)

- **Rust Library**: Core application logic and Tauri command handlers
- **Build Configuration**: Cargo manifest and build scripts
- **Capabilities**: Security and permission configuration via ACL manifests
- **Icons**: Application brand and window icons
- **Distribution**: Compiled binaries for Windows and macOS

## Key Features

- **Media Management**: Display and manage media library content
- **Content Details**: Rich information display for media items
- **Playback**: Integrated video player with modern controls
- **Settings**: Configurable application preferences
- **Dark Mode Support**: Theme switching capabilities
- **Cross-Platform Compatibility**: Consistent experience on macOS and Windows
- **Automatic Content Sharing**: Built-in sharing capabilities

## Development

- **Package Manager**: PNPM for efficient dependency management
- **Development Server**: Vite dev server with hot module reloading
- **Build Process**: TypeScript compilation + Vite bundling + Tauri build
- **Type Safety**: Full TypeScript support across frontend and Tauri bindings
- **Build Output**: Native executables for distribution

## Getting Started

Developers can explore the following chapters to:

1. Set up the development environment and prerequisites
2. Run the development server and test applications
3. Build for production distribution
4. Understand the architecture and component design
5. Contribute to the project following guidelines

For detailed information on specific topics, please refer to the chapters in this documentation.
