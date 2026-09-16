use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn content_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn is_duplicate(content: &str, existing_hashes: &[String]) -> bool {
    let hash = content_hash(content);
    existing_hashes.contains(&hash)
}