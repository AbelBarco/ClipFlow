# ClipFlow

> [!NOTE]
> 💻 **Current Compatibility:** Currently, ClipFlow is officially available for **Windows** (`.msi`) only. I am actively working on optimizing and testing the app to launch it on **macOS** and **Linux** very soon.

ClipFlow saves everything you copy (text, code, images, colors) and lets you retrieve it instantly with a global shortcut. Built with Rust + Tauri 2 and Svelte 5.

![ClipFlow Demo](https://github.com/clipflow/clipflow/raw/main/docs/demo.gif)

## Features

- 📋 **Full History** - Text, URLs, code, colors, images (max 500 items)
- ⚡ **Global Shortcut** - `Ctrl+Shift+V` (customizable) opens Spotlight window
- 🎨 **Color Detection** - Auto-detects HEX/RGB/HSL with live preview
- 🔍 **OCR** - Extract text from images (Vision/WinRT/Tesseract)
- 🔧 **Transformers** - Case conversion, encoding, formatting, slugify
- 🌐 **10 Languages** - Full UI translation (ES, EN, FR, DE, PT, IT, ZH, JA, KO, RU)
- 🛡️ **Privacy First** - Per-app exclusion via focused-window detection, secret heuristics with opt-out notice
- 🔒 **Optional Encryption** - XChaCha20-Poly1305 at rest, key in OS keychain
- 🔎 **Fuzzy Search** - Instant filtering with Fuse.js
- 🌙 **Theme** - Light/Dark/System automatic
- 📦 **Tiny** - <15 MB binary, <40 MB RAM
- 🖥️ **Cross-Platform** - Windows, macOS, Linux

## Quick Start

### Prerequisites

- Node.js 20+
- pnpm 9+
- Rust 1.75+
- Platform dependencies (see [Development](#development))

### Install

```bash
# Clone
git clone https://github.com/clipflow/clipflow.git
cd clipflow

# Install dependencies
pnpm install

# Development
pnpm tauri dev

# Build
pnpm tauri build
```

### Download

Pre-built binaries available on [Releases](https://github.com/clipflow/clipflow/releases).

| Platform              | Download                         |
| --------------------- | -------------------------------- |
| Windows (x64)         | `clipflow-windows-x86_64.msi`    |
| macOS (Intel)         | `clipflow-macos-x86_64.dmg`      |
| macOS (Apple Silicon) | `clipflow-macos-arm64.dmg`       |
| Linux (x64)           | `clipflow-linux-x86_64.AppImage` |

## Usage

1. **Start ClipFlow** - Runs in system tray
2. **Copy normally** - Everything is captured automatically
3. **Press `Ctrl+Shift+V`** - Opens Spotlight search window
4. **Type to filter** - Fuzzy search across all history
5. **Press `Enter`** - Pastes selected item into focused app
6. **Right-click tray** - Access main window, settings, quit

### Keyboard Shortcuts

| Shortcut       | Action                   |
| -------------- | ------------------------ |
| `Ctrl+Shift+V` | Open Spotlight (default) |
| `↑/↓`          | Navigate results         |
| `Enter`        | Paste selected           |
| `Esc`          | Close Spotlight          |
| `/`            | Focus search             |
| `Ctrl+Shift+U` | Uppercase transform      |
| `Ctrl+Shift+L` | Lowercase transform      |

## Transformers

Available via context menu or shortcuts:

| Transform     | Description           |
| ------------- | --------------------- |
| UPPERCASE     | Convert to uppercase  |
| lowercase     | Convert to lowercase  |
| Title Case    | Capitalize each word  |
| snake_case    | Convert to snake_case |
| kebab-case    | Convert to kebab-case |
| camelCase     | Convert to camelCase  |
| PascalCase    | Convert to PascalCase |
| Trim          | Remove whitespace     |
| Slugify       | URL-friendly slug     |
| JSON Pretty   | Format JSON           |
| JSON Minify   | Minify JSON           |
| URL Encode    | Encode for URLs       |
| URL Decode    | Decode from URLs      |
| Base64 Encode | Encode to Base64      |
| Base64 Decode | Decode from Base64    |

## Settings

Access via tray menu → Settings:

- **General**: Global shortcut, history limit, startup, notifications, theme
- **Language**: Interface language for the whole app (applied instantly)
- **Exclusions**: Focused-app ignore list (password managers excluded by default), secret-heuristic sensitivity, skip notice
- **Privacy**: Optional at-rest encryption (OS-keychain key)
- **OCR**: Enable/disable, language, auto-run on images

> **OCR notes (still fully offline):** on Windows it uses the built-in
> `Windows.Media.Ocr` engine — if recognition fails for a language, install
> its OCR pack in Settings → Time & Language → Language. On Linux it uses
> the `tesseract` CLI with the language selected in settings.

## Development

### Platform Dependencies

**Linux (Ubuntu/Debian)**

```bash
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev libssl-dev libsqlite3-dev tesseract-ocr libtesseract-dev
```

**macOS**

```bash
xcode-select --install
brew install tesseract
```

**Windows**

- Visual Studio 2022 with C++ workload
- WebView2 Runtime (included in Windows 11)

### Project Structure

```
ClipFlow/
├── src/                    # Frontend (Svelte 5)
│   ├── routes/             # Windows (main, spotlight, settings)
│   └── lib/features/       # Feature modules
│       ├── clipboard/      # History, search, items
│       ├── color/          # Detection, preview, conversion
│       ├── transformers/   # Text transformations
│       ├── settings/       # Configuration UI
│       └── ui/             # Shared components
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── commands/       # Tauri command handlers
│   │   ├── storage/        # SQLite + filesystem
│   │   ├── pipeline/       # Detection, dedupe, transform
│   │   ├── clipboard/      # Watcher, writer, exclusion
│   │   ├── ocr/            # Platform OCR providers
│   │   ├── hotkeys/        # Global shortcuts
│   │   ├── tray/           # System tray
│   │   ├── windows/        # Window management
│   │   └── config/         # Settings persistence
│   ├── migrations/         # SQL migrations
│   └── capabilities/       # Tauri permissions
├── tests/                  # Integration tests
├── docs/                   # Architecture, pipeline, security
├── scripts/                # Dev/build scripts
└── .github/workflows/      # CI/CD
```

### Commands

```bash
# Development
pnpm tauri dev              # Hot reload frontend + backend
pnpm dev                    # Frontend only (Vite)

# Code Quality
pnpm lint                   # ESLint
pnpm format                 # Prettier
pnpm svelte-check           # TypeScript

# Testing
pnpm test                   # Vitest (frontend)
cargo test                  # Rust tests (in src-tauri/)

# Build
pnpm tauri build            # Current platform
./scripts/build-all.sh      # All platforms (requires cross-compilation)
```

## Architecture

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation.

Key principles:

- **Layered**: Frontend ↔ Commands ↔ Pipeline/Storage/System
- **Pure Pipeline**: Detection, dedupe, transform are pure Rust
- **Capability-based**: Minimal permissions per window
- **Offline-first**: No network, no telemetry

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Run `pnpm lint && pnpm format && cargo clippy`
5. Submit a PR

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Framework
- [Svelte](https://svelte.dev/) - Reactive UI
- [Tailwind CSS](https://tailwindcss.com/) - Styling
- [Fuse.js](https://fusejs.io/) - Fuzzy search
- [Rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings

---

Made with ❤️ by the ClipFlow community
