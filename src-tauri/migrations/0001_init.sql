-- Initial schema for ClipFlow
CREATE TABLE IF NOT EXISTS clip_items (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    content TEXT NOT NULL,
    preview TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    size INTEGER NOT NULL,
    ocr_text TEXT,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_clip_items_timestamp ON clip_items(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_clip_items_type ON clip_items(type);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);