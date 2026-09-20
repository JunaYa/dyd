# DYD (Do Your Drawings)

DYD is a lightweight, cross-platform drawing application. It uses Excalidraw as the drawing engine and extends its functionality. It provides convenient local file management with a simple and quick user experience.

## Background

Why did I make Excalidraw into a desktop application?

- I believe the number of laptop users is gradually increasing.
- Simple and quick: I want it to be simple and quick to use, without complex operations, easy to open and edit on computers.
- Easy management: I want to easily manage local files.

## Project Goals

- Lightweight
- Cross-platform
- Easy to operate
- Simple

## Project Structure

- src-tauri: Desktop application written in Rust
- src: Web application written in React

## Local Development

Run `pnpm tauri dev` to launch the desktop app. The launcher starts Vite on port 1420, or the next available port, and passes the actual URL to Tauri. Hot reload shares that port. Exiting closes the server started by this launch without stopping other applications.

For the browser only, run `pnpm dev` and open the URL printed in the terminal. Other commands, including `pnpm tauri build`, pass through to the Tauri CLI. Automatic URL synchronization requires `pnpm tauri dev`; direct `cargo tauri dev` or `pnpm exec tauri dev` commands bypass the launcher.

Run `pnpm test:dev` to check port fallback, argument forwarding, and shutdown without compiling Rust.
