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
- **Optional encryption** (off by default, Settings → Privacy): `content` and
  `ocr_text` columns encrypted with XChaCha20-Poly1305; the 256-bit key lives
  in the OS keychain (DPAPI / Keychain / Secret Service), never on disk.
  Enabling/disabling migrates every row and only then persists the flag, so
  the setting and the database cannot disagree.
- Without encryption, protection relies on OS file permissions and
  full-disk encryption.
- Configurable max history (default 500 items)

### In Memory

- Clipboard content held briefly in Rust strings
- No logging of clipboard content (only ids, sizes and exclusion reasons)

### In Transit

- No network communication
- IPC via Tauri (local only)

## Exclusion Mechanisms

### 1. Application Exclusion List

Clipboard content copied while a listed application is focused is ignored:

- Default: 1Password, Bitwarden, LastPass, KeePass
- User can add/remove apps
- Matched case-insensitively against process name, OS app name and window title
- Focus detection: `GetForegroundWindow` (Windows),
  `NSWorkspace.frontmostApplication` (macOS), X11 active window plus native
  Hyprland/KWin Wayland support (Linux, via `active-win-pos-rs`)
- On compositors without a focus API the query returns "unknown" and only
  content heuristics (below) apply — this degradation is logged, never silent

### 2. Concealed Type Detection

The `respectConcealed` toggle gates the content heuristics below. Note: no
OS concealed-clipboard formats (`CFSTR_CONCEALED`, `NSPasteboardTypeConcealed`)
are read — clipboard access goes through `arboard` (text/image only) — so
"concealed" here means heuristic detection, and the toggle is its on/off switch.

### 3. Content Heuristics

Automatic detection of likely secrets, in two sensitivity levels
(Settings → Excluded applications):

- `conservative` (default): password indicators, known secret markers
  (`sk_live_`, `ghp_`, `-----BEGIN`, …), single-line blobs ≥32 chars with
  entropy > 5.5 bits/char
- `standard` (legacy): single-line blobs 20–200 chars with entropy > 4.5
- Both levels allow-list common non-secrets so they are never dropped:
  UUIDs, 7–128 char hex (git SHAs), JWT-shaped tokens, Stripe publishable
  (`pk_live_/pk_test_`) and object (`cus_/pi_/…`) IDs
- Heuristic drops are **not silent**: one rate-limited OS notification
  ("possible secret skipped", localized, opt-out via `notifyOnExclude`,
  silenced when notifications are off)

### 4. Password Field Detection

Covered by layers 1–3 combined: password-manager focus exclusion plus
`password`-indicator and secret-pattern heuristics on content. There is no
separate OS password-field API integration.

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
- Input synthesis (Ctrl/Cmd+V after an explicit user pick; never
  keylogs, never types without a user action — see `writer.rs`)

### Settings Window

- Clipboard read (for testing shortcuts)
- File system (app data only)
- SQL database (read/write config)
- Window management

## Platform-Specific Considerations

### Windows

- Clipboard polling via `arboard` (500ms text, ~2s images)
- Global shortcuts via `RegisterHotKey`
- Tray via `Shell_NotifyIcon`
- Exclusion checks `GetForegroundWindow` owning process (see above)

### macOS

- Clipboard polling via `arboard` (500ms text, ~2s images)
- Global shortcuts via `CGEventTap` (requires Accessibility permission)
- Tray via `NSStatusItem`
- Exclusion checks `NSWorkspace.frontmostApplication` (see above)

### Linux

- Clipboard polling via `arboard` (500ms text, ~2s images); native
  Wayland portal support is planned (see ROADMAP — polling is unreliable
  on compositors without XWayland)
- Global shortcuts vary by DE (may require portal)
- Tray via `libayatana-appindicator` or `StatusNotifierItem`
- Exclusion via X11 active window or native Hyprland/KWin APIs (see above)

## Privacy

- **No Telemetry**: No usage data collected
- **No Network**: Zero network requests
- **No Auto-Updates**: User controls updates
- **Local-First**: All data stays on device
- **Open Source**: Auditable codebase

## Secure Development

### Dependencies

- Pinned versions in `Cargo.lock`
- `cargo clippy -- -D warnings` and `cargo fmt --check` enforced in CI
- New native claims must ship with tests (see `exclusion`, `crypto`,
  `repository` unit tests)

### Code Practices

- No `unwrap()`/`expect()` on external input
- Explicit error handling with `anyhow`
- No `unsafe` except FFI boundaries
- Input validation at command boundaries

### Testing

- Unit tests for exclusion heuristics, crypto round-trips and
  duplicate-refresh (`cargo test`), enforced in CI on every push/PR
- Frontend i18n completeness tests (`vitest`)
- Fuzzing for parser functions (planned, see ROADMAP)

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
