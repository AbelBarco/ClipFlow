# ClipFlow Roadmap

## Version 0.1 (MVP) - Current

- [x] Clipboard watcher (500ms text polling, ~2s image sampling)
- [x] History persistence (SQLite, 500 items max)
- [x] Global shortcut → Spotlight window
- [x] Spotlight window is draggable and remembers its position
- [x] Type detection (text/url/code/color/image)
- [x] Color preview and conversion
- [x] OCR (native on-device APIs: WinRT on Windows, Tesseract on Linux)
- [x] Plain text paste
- [x] Transformers (case, encoding, formatting)
- [x] Per-app exclusion via focused-window detection (Win/macOS/X11, Hyprland/KWin)
- [x] Secret heuristics (sensitivity levels, non-secret allow-list, skip notice)
- [x] Optional at-rest encryption (XChaCha20-Poly1305, OS-keychain key)
- [x] Duplicate re-copy refreshes timestamp (moves back to top)
- [x] Fuzzy search (Fuse.js)
- [x] System tray with menu
- [x] Settings window
- [x] Light/dark theme (system)
- [x] Localization: full UI in 10 languages (ES, EN, FR, DE, PT, IT, ZH, JA, KO, RU)
- [x] Native offline spell checker (all 10 languages, auto-fix)
- [x] CI gate on every push/PR (eslint, prettier, svelte-check, clippy, fmt, tests)
- [x] Cross-platform (Win/macOS/Linux)

## Version 0.2 (Polish & Reliability)

- [ ] **Clipboard formats**: Rich text (RTF), HTML, file paths
- [ ] **Image handling**: Thumbnails, multiple images, drag-drop
- [ ] **Search improvements**: Filters (type, date, app), keyboard nav
- [x] **History fetch**: Bounded by history setting, single-row lookup for paste/delete/preview
- [ ] **Performance**: Virtualized list, debounced search, lazy OCR
- [ ] **Reliability**: Better error recovery, watchdog for watcher
- [ ] **Accessibility**: Full keyboard nav, screen reader support
- [ ] **Linux Wayland**: Native clipboard portal support (**high priority** — polling is unreliable on Wayland compositors without XWayland/Hyprland/KWin APIs)
- [ ] **Auto-update**: Tauri updater integration
- [ ] **Crash reporting**: Optional, opt-in

## Version 0.3 (Power User Features)

- [ ] **Pinning**: Pin items to top of history
- [ ] **Tags/Categories**: User-defined tags for items
- [ ] **Collections**: Named groups of clips
- [ ] **Snippets**: Save frequently used text as snippets
- [ ] **Sync** (optional): Encrypted sync via user's cloud (iCloud/OneDrive/Drive)
- [ ] **Plugin API**: Lua/WASM plugins for custom transformers
- [ ] **Scripting**: Trigger actions on clipboard events
- [ ] **Keyboard shortcuts**: Customizable per-action shortcuts

## Version 0.4 (Team & Enterprise)

- [ ] **Shared histories**: Team clipboard sharing (E2E encrypted)
- [ ] **Admin controls**: Policy management, exclusion enforcement
- [ ] **Audit log**: Optional clipboard access logging
- [ ] **SSO/SCIM**: Enterprise identity integration
- [ ] **Managed config**: MDM-deployed settings

## Version 1.0 (Stable Release)

- [ ] **Stable API**: Semantic versioning commitment
- [ ] **Long-term support**: 12-month maintenance window
- [ ] **Signed releases**: All platforms with verified signatures
- [ ] **Store distribution**: Microsoft Store, Mac App Store, Snap, Flathub
- [ ] **Documentation**: Complete user and developer docs
- [x] **Localization**: Full UI in 10 languages (ES, EN, FR, DE, PT, IT, ZH, JA, KO, RU)

## Future Ideas (Post-1.0)

- [ ] **AI-powered**: Smart categorization, PII detection, summarization
- [ ] **Browser extension**: Direct clipboard access from web
- [ ] **Mobile companion**: iOS/Android app for cross-device sync
- [ ] **Clipboard history timeline**: Visual timeline view
- [ ] **OCR improvements**: Table detection, handwriting, multi-language, macOS Vision backend
- [ ] **Code intelligence**: Syntax highlighting, language detection, formatting
- [ ] **Workflow automation**: Trigger scripts/workflows on patterns

## Technical Debt

- [ ] **Test coverage**: Target >80% for pipeline, >60% overall
- [ ] **Benchmark suite**: Automated performance regression detection
- [ ] **Fuzzing**: Continuous fuzzing for parsers
- [ ] **Dependency audit**: Automated weekly checks
- [ ] **Architecture docs**: Keep ARCHITECTURE.md current

## Platform-Specific

### Windows

- [ ] MSIX packaging for Store
- [ ] WinUI 3 native controls (optional)
- [ ] Better high-DPI handling

### macOS

- [ ] Universal binary (Intel + Apple Silicon)
- [ ] Notarization and stapling
- [ ] Touch Bar support (legacy)
- [ ] Shortcuts app integration

### Linux

- [ ] Flatpak manifest
- [ ] Snapcraft.yaml
- [ ] AppImage build
- [ ] DE-specific tray (GNOME, KDE, etc.)
- [ ] systemd user service for background watcher

## Community

- [ ] Contributing guide
- [ ] Code of conduct
- [ ] Issue templates
- [ ] Discord/Matrix community
- [ ] Plugin/template gallery
