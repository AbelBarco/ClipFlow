-- Add OCR text column (if not exists from initial migration)
-- This migration ensures the column exists
ALTER TABLE clip_items ADD COLUMN IF NOT EXISTS ocr_text TEXT;

-- Add index for OCR search
CREATE INDEX IF NOT EXISTS idx_clip_items_ocr_text ON clip_items(ocr_text);