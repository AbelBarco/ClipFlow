use std::path::PathBuf;

const LANGS: &[&str] = &["es", "en", "fr", "de", "pt", "it", "ru"];

fn main() {
    tauri_build::build();

    // Compile the curated, auditable word lists (dict-src/{lang}.txt,
    // "word count" per line) into compact FST maps (dicts/{lang}.fst) that
    // ship as Tauri resources for the offline spell checker.
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = manifest.join("dicts");
    std::fs::create_dir_all(&out_dir).expect("create dicts dir");

    for lang in LANGS {
        let src = manifest.join(format!("dict-src/{lang}.txt"));
        println!("cargo:rerun-if-changed=dict-src/{lang}.txt");
        let dst = out_dir.join(format!("{lang}.fst"));
        // Skip rebuild when the artifact is newer than its source.
        if let (Ok(src_meta), Ok(dst_meta)) = (std::fs::metadata(&src), std::fs::metadata(&dst)) {
            if let (Ok(src_t), Ok(dst_t)) = (src_meta.modified(), dst_meta.modified()) {
                if dst_t >= src_t {
                    continue;
                }
            }
        }
        let text = std::fs::read_to_string(&src).unwrap_or_else(|e| panic!("read {lang}.txt: {e}"));
        // fst::MapBuilder requires lexicographic insertion; frequencies ride
        // along as u64 values for suggestion ranking.
        let mut entries: Vec<(String, u64)> = Vec::new();
        for line in text.lines() {
            let mut parts = line.rsplitn(2, ' ');
            let count: u64 = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0);
            let word = parts.next().unwrap_or("").trim();
            if word.is_empty() || count == 0 {
                continue;
            }
            entries.push((word.to_string(), count));
        }
        entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        entries.dedup_by(|a, b| a.0 == b.0);

        let file = std::fs::File::create(&dst).unwrap_or_else(|e| panic!("create {lang}.fst: {e}"));
        let mut builder = fst::MapBuilder::new(file).expect("fst builder");
        for (word, count) in &entries {
            builder.insert(word, *count).expect("fst insert");
        }
        builder.finish().expect("fst finish");
        println!(
            "cargo:warning=compiled spell dictionary {lang}: {} words",
            entries.len()
        );
    }
}
