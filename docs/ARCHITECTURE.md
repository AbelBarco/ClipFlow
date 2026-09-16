# ClipFlow Architecture

## Overview

ClipFlow follows a layered architecture with clear separation between frontend (UI) and backend (system integration).

```
┌─────────────────────────────────────────────────────────────┐
│                      Frontend (Svelte 5)                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │   Main      │ │  Spotlight  │ │  Settings   │  Windows  │
│  │  Window     │ │  Window     │ │  Window     │           │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘           │
│         │               │               │                   │
│         └───────────────┼───────────────┘                   │
│                         ▼                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Feature Modules (lib/features)         │   │
│  │  ┌─────────┐ ┌───────┐ ┌───────────┐ ┌───────────┐  │   │
│  │  │Clipboard│ │ Color │ │Transformers│ │ Settings  │  │   │
│  │  └────┬────┘ └───┬───┘ └─────┬─────┘ └─────┬─────┘  │   │
│  └───────┼──────────┼──────────┼──────────┼──────────┘   │
│          │          │          │          │               │
│          ▼          ▼          ▼          ▼               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Tauri API (invoke)                      │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Backend (Rust)                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                  Command Layer                       │   │
│  │  Thin wrappers that delegate to pipeline/storage    │   │
│  └─────────────────────────────────────────────────────┘   │
│                              │                              │
│          ┌───────────────────┼───────────────────┐          │
│          ▼                   ▼                   ▼          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │  Pipeline   │    │  Storage    │    │   System    │    │
│  │ (Pure Rust) │    │  (SQLite)   │    │  Integration│    │
│  └─────────────┘    └─────────────┘    └─────────────┘    │
│          │                   │                   │          │
│          ▼                   ▼                   ▼          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │• Detector   │    │• Repository │    │• Clipboard  │    │
│  │• Dedupe     │    │• Rotation   │    │• Hotkeys    │    │
│  │• Color      │    │• Image Store│    │• Tray       │    │
│  │• Transform  │    │             │    │• Windows    │    │
│  └─────────────┘    └─────────────┘    └─────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Layers

### Frontend (Svelte 5 + TypeScript)

- **Routes**: Each window is a SvelteKit route (`main`, `spotlight`, `settings`)
- **Features**: Colocated feature modules with components, stores, and API
- **Stores**: Svelte 5 runes for reactive state management
- **Components**: Reusable UI components per feature

### Backend (Rust)

#### Command Layer (`src/commands/`)
Thin wrappers that:
- Receive invoke calls from frontend
- Validate input
- Delegate to pipeline/storage/system modules
- Return serializable responses
- No business logic

#### Pipeline (`src/pipeline/`)
Pure Rust, no Tauri dependencies:
- **Detector**: Classifies clipboard content (text/url/color/code/image)
- **Dedupe**: Content hashing for duplicate detection
- **Color Parser**: Parse/convert HEX, RGB, HSL formats
- **Transformers**: Text transformations (case, encoding, formatting)

#### Storage (`src/storage/`)
- **Database**: SQLite via `tauri-plugin-sql` with migrations
- **Repository**: CRUD operations for clipboard items
- **Rotation**: FIFO logic to maintain max 500 items
- **Image Store**: Filesystem storage for images

#### System Integration
- **Clipboard Watcher**: 300ms polling with platform-specific APIs
- **Clipboard Writer**: Multi-MIME clipboard writing
- **Exclusion**: Detect password managers and concealed input
- **OCR**: Platform-native OCR (Vision/WinRT/Tesseract)
- **Hotkeys**: Global shortcut registration
- **Tray**: System tray icon and menu
- **Windows**: Window management (show/hide/position)

## Data Flow

### Clipboard Capture
```
OS Clipboard → Watcher (300ms poll) → Detect Type → Hash Content
                                    ↓
                            Check Exclusions → Check Duplicates
                                    ↓
                            Process Pipeline → Store in SQLite
                                    ↓
                            Emit Event → Frontend Updates
```

### Spotlight Search
```
User Input → Fuse.js Fuzzy Search → Filtered Results → Render List
                                    ↓
                            Select Item → Invoke Paste Command
                                    ↓
                            Write to Clipboard → Hide Window
```

### Color Detection
```
Clipboard Text → Color Detector → Parse Format → Convert All Formats
                                    ↓
                            Store with Type: "color" → Render Preview
```

## Concurrency Model

- **Frontend**: Single-threaded event loop (Svelte reactivity)
- **Backend**: Tokio async runtime
- **Database**: WAL mode for concurrent reads/writes
- **Clipboard Watcher**: Dedicated tokio task with interval
- **OCR**: Async tasks to avoid blocking

## Security

- **Exclusion**: Automatic detection of password managers
- **Concealed Types**: Respect OS-level concealed input flags
- **Permissions**: Granular Tauri capabilities per window
- **No Network**: Fully offline, no telemetry
- **Local Storage**: SQLite in app data directory

## Performance Targets

- **RAM**: < 40 MB baseline
- **Binary Size**: < 15 MB (stripped)
- **Startup**: < 500ms cold start
- **Search**: < 10ms for 500 items (Fuse.js)
- **Clipboard Poll**: 300ms interval, < 1ms processing