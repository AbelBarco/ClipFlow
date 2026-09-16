# ClipFlow Roadmap

## Version 0.1 (MVP) - Current
- [x] Clipboard watcher (300ms polling)
- [x] History persistence (SQLite, 500 items max)
- [x] Global shortcut → Spotlight window
- [x] Type detection (text/url/code/color/image)
- [x] Color preview and conversion
- [x] OCR (native APIs)
- [x] Plain text paste
- [x] Transformers (case, encoding, formatting)
- [x] Password manager exclusion
- [x] Fuzzy search (Fuse.js)
- [x] System tray with menu
- [x] Settings window
- [x] Light/dark theme (system)
- [x] Cross-platform (Win/macOS/Linux)

## Version 0.2 (Polish & Reliability)
- [ ] **Clipboard formats**: Rich text (RTF), HTML, file paths
- [ ] **Image handling**: Thumbnails, multiple images, drag-drop
- [ ] **Search improvements**: Filters (type, date, app), keyboard nav
- [ ] **Performance**: Virtualized list, debounced search, lazy OCR
- [ ] **Reliability**: Better error recovery, watchdog for watcher
- [ ] **Accessibility**: Full keyboard nav, screen reader support
- [ ] **Linux Wayland**: Native clipboard portal support
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
- [ ] **Localization**: i18n support (start with EN, ES, FR, DE, ZH, JA)

## Future Ideas (Post-1.0)
- [ ] **AI-powered**: Smart categorization, PII detection, summarization
- [ ] **Browser extension**: Direct clipboard access from web
- [ ] **Mobile companion**: iOS/Android app for cross-device sync
- [ ] **Clipboard history timeline**: Visual timeline view
- [ ] **OCR improvements**: Table detection, handwriting, multi-language
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