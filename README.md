# ClipFlow ⚡

> Ultra-lightweight, cross-platform, speed-focused clipboard manager. Lives in your system tray and instantly restores anything you copy.

**ClipFlow** is a modern alternative to traditional clipboard managers. Designed with a *Spotlight-style* fast-access interface triggered by a global hotkey, it delivers flawless native performance using **under 40 MB of RAM** with a binary size of **less than 15 MB**.


## 🌟 Key Features

* **🚀 Lightning Fast & Lightweight:** Minimal memory footprint (<40 MB RAM) and small binary size (<15 MB).
* **🎯 Spotlight-Style Interface:** Floating launcher accessible anytime with a customizable global hotkey.
* **🧠 Smart Type Detection:** Auto-categorizes text, URLs, colors, code snippets, and images.
* **🔒 Privacy-First:** Automatically skips concealed types from password managers (like 1Password or Bitwarden).
* **🎨 Color Inspector:** Previews and converts `HEX`, `RGB`, and `HSL` color values on the fly.
* **👁️ Native OCR:** Extracts text from copied images using native OS APIs (macOS Vision Framework, Windows WinRT) with Tesseract fallback on Linux.
* **⚡ Text Transformers:** Case conversion (`UPPERCASE`, `lowercase`, `camelCase`, `snake_case`), JSON formatting, trimming, and slug generation.
* **🔍 Fuzzy Search:** Instant searching across your clipboard history using `Fuse.js`.

* ## 🛠️ Tech Stack

* **Backend:** Rust + Tauri 2
* **Frontend:** Svelte 5 (Runes) + TypeScript + Tailwind CSS
* **Database:** SQLite + Local File Storage for media assets
