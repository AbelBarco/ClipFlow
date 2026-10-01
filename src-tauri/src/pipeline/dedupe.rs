use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn content_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Stable (cross-restart) 64-bit FNV-1a hash, hex-encoded. Used for delete
/// tombstones, which must survive app restarts — unlike `content_hash`
/// (SipHash with per-process random keys, only valid in-memory).
pub fn stable_content_hash(content: &str) -> String {
    format!("{:016x}", fnv1a64(content.as_bytes()))
}

/// Stable image fingerprint mirroring the watcher's sampling (dimensions +
/// length + head/tail bytes) so a deleted image stays deleted across boots
/// even though re-saving it produces a different file path.
pub fn stable_image_hash(width: usize, height: usize, rgba: &[u8]) -> String {
    let mut h = fnv1a64(&(width as u64).to_le_bytes());
    h = fnv1a64_combine(h, &(height as u64).to_le_bytes());
    h = fnv1a64_combine(h, &(rgba.len() as u64).to_le_bytes());
    h = fnv1a64_combine(h, &rgba[..rgba.len().min(32 * 1024)]);
    if rgba.len() > 32 * 1024 {
        h = fnv1a64_combine(h, &rgba[rgba.len() - 4096..]);
    }
    format!("{h:016x}")
}

fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn fnv1a64_combine(mut hash: u64, data: &[u8]) -> u64 {
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn is_duplicate(content: &str, existing_hashes: &[String]) -> bool {
    let hash = content_hash(content);
    existing_hashes.contains(&hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_hashes_are_deterministic() {
        assert_eq!(stable_content_hash("hola"), stable_content_hash("hola"));
        assert_ne!(stable_content_hash("hola"), stable_content_hash("hola "));
        assert_ne!(stable_content_hash("hola"), stable_content_hash("adios"));
        assert_eq!(stable_content_hash("x").len(), 16);
        let img = stable_image_hash(2, 2, &[1u8; 16]);
        assert_eq!(img, stable_image_hash(2, 2, &[1u8; 16]));
        assert_ne!(img, stable_image_hash(2, 2, &[2u8; 16]));
        assert_ne!(img, stable_image_hash(3, 2, &[1u8; 24]));
    }
}
