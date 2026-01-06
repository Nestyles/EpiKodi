# EpiKodi Contributor Guide

## Introduction

### What is EpiKodi?

EpiKodi is the alternative to Kodi. It's a media station that allows you to have details, play and share content automatically.

The development of EpiKodi is still in its early stages, so please bare with us as we build out this guide.

### What is this guide?

This guide is for anyone who wants to contribute to EpiKodi. It's a work in progress, and will be updated regularly.

### How can I contribute?

There are many ways to contribute to Cap. You can:

- [Report a bug](https://github.com/Nestyles/EpiKodi/issues/new)
- Submit a PR

## Running EpiKodi

### Development Requirements

Before anything else, make sure you have the following installed:

- Node Version 20+
- Rust 1.88.0+
- pnpm/npm 8.10.5+

### General Setup

Run `pnpm install`, to install dependencies.
Run `pnpm tauri dev`, to start the application.

#### Where are my datas stored?

You can find your datas at `%appdata%/epikodi/`.
