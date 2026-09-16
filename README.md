# ClipFlow

Lightweight cross-platform clipboard manager that lives in your system tray.

## Features

- **History**: Persistent clipboard history (max 500 items, FIFO rotation)
- **Spotlight**: Global shortcut → floating search window
- **Types**: Auto-detects text, URLs, code, colors, images, files
- **Colors**: HEX/RGB/HSL detection with visual preview & conversion
- **OCR**: Local text extraction from images (native APIs)
- **Transforms**: 15 built-in text transformations
- **Exclusions**: Auto-ignore password managers
- **Search**: Fuzzy search via Fuse.js
- **Tray**: Native system tray with quick actions

## Tech Stack

- **Backend**: Rust + Tauri 2
- **Frontend**: Svelte 5 (runes) + TypeScript + Tailwind CSS + Vite
- **Storage**: SQLite (tauri-plugin-sql) + filesystem for images
- **Search**: Fuse.js (fuzzy search)
- **OCR**: Vision (macOS), WinRT (Windows), Tesseract (Linux)

## Quick Start

```bash
# Prerequisites: Node.js 20+, Rust 1.75+, pnpm
git clone https://github.com/AbelBarco/ClipFlow
cd clipflow
pnpm install
pnpm tauri dev
```

## Building

```bash
# Development
pnpm tauri dev

# Production build
pnpm tauri build

# Cross-platform (Linux only)
./scripts/build-all.sh
```

## Configuration

Settings managed via Settings window:
- Global shortcut (default: `CmdOrCtrl+Shift+V`)
- Max history items (50-5000)
- Excluded applications
- OCR enable/disable & language
- Theme (light/dark/system)
- Launch at startup
- Show in tray

## Keyboard Shortcuts (Spotlight)

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate |
| `Enter` | Paste selected |
| `Cmd+Enter` | Paste as plain text |
| `Esc` | Close window |

## Project Structure

```
ClipFlow/
├── src/                    # Frontend (SvelteKit)
│   ├── routes/            # Pages (main, spotlight, settings)
│   └── lib/features/      # Feature modules
│       ├── clipboard/     # History, search, items
│       ├── color/         # Color detection & preview
│       ├── transformers/  # Text transformations
│       ├── settings/      # Configuration UI
│       └── ui/            # Shared components
├── src-tauri/             # Backend (Rust)
│   ├── src/
│   │   ├── commands/      # Tauri command handlers
│   │   ├── pipeline/      # Detection, dedupe, colors, transforms
│   │   ├── storage/       # SQLite, repository, rotation, images
│   │   ├── clipboard/     # Watcher, writer, exclusions
│   │   ├── ocr/           # Platform OCR providers
│   │   ├── hotkeys/       # Global shortcut
│   │   ├── tray/          # System tray
│   │   ├── windows/       # Window management
│   │   └── config/        # App configuration
│   ├── capabilities/      # Per-window permissions
│   └── migrations/        # SQL schema versions
├── docs/                  # Architecture, pipeline, security, roadmap
├── tests/                 # Integration tests
└── scripts/               # Dev & build scripts
```
