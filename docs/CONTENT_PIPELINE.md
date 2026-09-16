# Content Pipeline

## Overview

The content pipeline processes all clipboard data through a series of stages:
**Detect → Dedupe → Classify → Store → Notify**

Each stage is a pure Rust function with no side effects, making them easily testable.

## Stage 1: Detection (`detector.rs`)

Classifies clipboard content based on MIME type and content analysis.

### Input
- Raw clipboard bytes
- Optional MIME type from clipboard

### Detection Logic

```
1. Check MIME type
   ├── image/* → "image"
   └── text/* or application/json → Continue to content analysis

2. Content Analysis (trimmed)
   ├── Color format? → "color"
   │   ├── #RGB, #RGBA, #RRGGBB, #RRGGBBAA
   │   ├── rgb(r, g, b)
   │   ├── rgba(r, g, b, a)
   │   ├── hsl(h, s%, l%)
   │   └── hsla(h, s%, l%, a)
   ├── URL pattern? → "url"
   │   └── ^https?://[^\s/$.?#].[^\s]*$
   ├── Code indicators? → "code"
   │   ├── Keywords: fn, function, const, let, class, etc.
   │   ├── High density of: {}[]();:=<>
   │   └── Multi-line with indentation
   └── Default → "text"
```

### Output
```rust
pub struct ClipItem {
    id: String,           // UUID v4
    r#type: String,       // "text" | "url" | "code" | "color" | "image"
    content: String,      // Original content
    preview: String,      // Truncated for display
    timestamp: i64,       // Unix ms
    size: i64,            // Byte size
    ocr_text: Option<String>,
    metadata: Option<serde_json::Value>,
}
```

## Stage 2: Deduplication (`dedupe.rs`)

Prevents storing duplicate clipboard content.

### Algorithm
- FNV-1a 64-bit hash of normalized content
- Normalization: trim whitespace, normalize line endings
- Check against recent hashes (in-memory LRU, 1000 entries)
- Also check database for persistence across restarts

### Hash Function
```rust
fn content_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
```

## Stage 3: Classification & Enrichment

### Color Enrichment
If type is "color", parse and convert to all formats:
- HEX: `#RRGGBB` or `#RRGGBBAA`
- RGB: `rgb(r, g, b)` or `rgba(r, g, b, a)`
- HSL: `hsl(h, s%, l%)` or `hsla(h, s%, l%, a)`
- CSS: Modern `rgb(r g b / a)` syntax

### Image Handling
- Save to filesystem: `$APPDATA/clipflow/images/{uuid}.{ext}`
- Store path in `content` field
- Generate thumbnail (200x200 max)
- Queue for OCR if enabled

### Code Detection Enhancement
- Detect language from shebang, file extension, or keywords
- Store language in metadata for syntax highlighting

## Stage 4: Storage (`repository.rs`)

### Database Schema
```sql
CREATE TABLE clip_items (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    content TEXT NOT NULL,
    preview TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    size INTEGER NOT NULL,
    ocr_text TEXT,
    metadata TEXT
);

CREATE INDEX idx_clip_items_timestamp ON clip_items(timestamp DESC);
CREATE INDEX idx_clip_items_type ON clip_items(type);
```

### Operations

**Insert**
```rust
INSERT INTO clip_items (id, type, content, preview, timestamp, size, ocr_text, metadata)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);
```

**Query (with pagination)**
```rust
SELECT * FROM clip_items 
ORDER BY timestamp DESC 
LIMIT 500;
```

**Rotation (FIFO)**
```rust
DELETE FROM clip_items 
WHERE id IN (
    SELECT id FROM clip_items 
    ORDER BY timestamp ASC 
    LIMIT ?
);
```

## Stage 5: Notification

Emit Tauri event to frontend:
```rust
app.emit("clipboard-item-added", &clip_item)?;
```

Frontend receives via:
```typescript
// In clipboard store
app.listen("clipboard-item-added", (event) => {
    clipboardStore.addItem(event.payload);
});
```

## OCR Pipeline (Async, Optional)

Triggered when:
- Image added and `ocr.autoRun = true`
- User manually requests OCR on image item

```
Image Path → Platform OCR Provider → Extracted Text → Update Database → Notify Frontend
```

### Providers
- **macOS**: Vision Framework (`VNRecognizeTextRequest`)
- **Windows**: WinRT `Windows.Media.Ocr`
- **Linux**: Tesseract CLI (`tesseract stdout -l eng`)

## Error Handling

Each stage returns `Result<T, String>`:
- Detection errors → Default to "text"
- Dedupe errors → Allow insert (fail open)
- Storage errors → Log and retry
- OCR errors → Store empty, allow manual retry

## Testing

Unit tests for each stage:
- `detector.rs`: All type detection cases
- `dedupe.rs`: Hash consistency, collision handling
- `color_parser.rs`: Round-trip conversions
- `transformers.rs`: All transformation variants

Integration tests:
- Full pipeline with sample data
- Database CRUD + rotation
- Cross-platform clipboard formats