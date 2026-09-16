# Security Model

## Threat Model

ClipFlow handles sensitive clipboard data. We assume:
- **User Device**: Trusted (user controls the machine)
- **Clipboard Content**: May contain secrets, PII, credentials
- **Other Applications**: May be malicious or compromised
- **Network**: Not used (fully offline)

## Data Protection

### At Rest
- SQLite database in app data directory (OS-protected)
- Images stored in app data subdirectory
- No encryption (relies on OS file permissions and full-disk encryption)
- Configurable max history (default 500 items)

### In Memory
- Clipboard content held briefly in Rust strings
- Zeroized on drop where possible
- No logging of clipboard content

### In Transit
- No network communication
- IPC via Tauri (local only)

## Exclusion Mechanisms

### 1. Application Exclusion List
Configurable list of application names whose clipboard data is ignored:
- Default: 1Password, Bitwarden, LastPass, KeePass
- User can add/remove apps
- Matched against active window title/process name

### 2. Concealed Type Detection
Respects OS-level "concealed" clipboard formats:
- **Windows**: `CFSTR_CONCEALED` format
- **macOS**: `NSPasteboardTypeConcealed` 
- **Linux**: No standard, heuristic-based

### 3. Content Heuristics
Automatic detection of likely secrets:
- High entropy strings (> 4.5 bits/char)
- Common patterns: `api_key`, `secret`, `token`, `password`
- Known prefixes: `ghp_`, `sk_live`, `pk_live`, `-----BEGIN`

### 4. Password Field Detection
- Window title contains "password", "login", "sign in"
- Active element is password input type (where detectable)

## Permissions (Tauri Capabilities)

### Main Window
- Clipboard read/write
- File system (app data only)
- SQL database
- Global shortcut
- Notifications
- Tray
- Window management

### Spotlight Window
- Clipboard read/write
- SQL database (read-only for search)
- Global shortcut (receive only)
- Window management (show/hide/focus)

### Settings Window
- Clipboard read (for testing shortcuts)
- File system (app data only)
- SQL database (read/write config)
- Window management

## Platform-Specific Considerations

### Windows
- Uses `OpenClipboard`/`GetClipboardData` with `CF_UNICODETEXT`
- Global shortcuts via `RegisterHotKey`
- Tray via `Shell_NotifyIcon`
- Checks `GetForegroundWindow` process for exclusion

### macOS
- Uses `NSPasteboard` APIs
- Global shortcuts via `CGEventTap` (requires Accessibility permission)
- Tray via `NSStatusItem`
- Checks `NSWorkspace.frontmostApplication` for exclusion

### Linux
- Uses `wl-clipboard` (Wayland) or `xclip` (X11)
- Global shortcuts vary by DE (may require portal)
- Tray via `libayatana-appindicator` or `StatusNotifierItem`
- Exclusion via `xdotool` or similar (best effort)

## Privacy

- **No Telemetry**: No usage data collected
- **No Network**: Zero network requests
- **No Auto-Updates**: User controls updates
- **Local-First**: All data stays on device
- **Open Source**: Auditable codebase

## Secure Development

### Dependencies
- Minimal dependency tree
- Regular `cargo audit` checks
- Pinned versions in `Cargo.lock`
- `deny` warnings in CI

### Code Practices
- No `unwrap()`/`expect()` on external input
- Explicit error handling with `anyhow`
- No `unsafe` except FFI boundaries
- Input validation at command boundaries

### Testing
- Fuzzing for parser functions
- Property-based tests for transformations
- Integration tests for exclusion logic

## Incident Response

If a vulnerability is discovered:
1. Report via GitHub Security Advisories
2. Fix in main branch
3. Release patch version
4. Publish security advisory
5. Users update via their preferred method

## Compliance

- **GDPR**: No personal data processing (local only)
- **CCPA**: No data sale/sharing
- **SOC 2**: Not applicable (no service)
- **HIPAA**: Not a covered entity