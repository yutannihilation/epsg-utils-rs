use std::fs;
use std::io::Write;
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;

fn main() {
    let wkt_dir = Path::new("data-raw/EPSG-v12_054-WKT");

    // Collect all EPSG-CRS-*.wkt files
    let mut files: Vec<_> = fs::read_dir(wkt_dir)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", wkt_dir.display()))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().into_string().ok()?;
            if name.starts_with("EPSG-CRS-") && name.ends_with(".wkt") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    files.sort();

    println!("Found {} CRS files", files.len());

    let mut wkt_lines = String::new();
    let mut json_lines = String::new();
    let mut count = 0u32;
    let mut skipped = 0u32;

    for filename in &files {
        let path = wkt_dir.join(filename);
        let wkt = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
        let wkt = wkt.trim();

        // Only process PROJCRS definitions
        if !wkt.starts_with("PROJCRS[") {
            skipped += 1;
            continue;
        }

        let crs = match epsg_utils::parse_wkt2(wkt) {
            Ok(crs) => crs,
            Err(e) => {
                eprintln!("  WARN: skipping {} (parse error: {e})", filename);
                skipped += 1;
                continue;
            }
        };

        let projjson = serde_json::to_string(&crs.to_projjson())
            .unwrap_or_else(|e| panic!("Failed to serialize {}: {e}", path.display()));

        let code: i32 = filename
            .strip_prefix("EPSG-CRS-")
            .and_then(|s| s.strip_suffix(".wkt"))
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| panic!("Cannot extract EPSG code from {filename}"));

        // Tab-separated: code\tstring\n
        wkt_lines.push_str(&format!("{code}\t{wkt}\n"));
        json_lines.push_str(&format!("{code}\t{projjson}\n"));
        count += 1;
    }

    // Compress and write
    let write_compressed = |path: &str, data: &str| {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(data.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();
        let uncompressed_size = data.len();
        fs::write(path, &compressed).unwrap();
        println!(
            "Generated {path} ({} -> {} bytes, {:.0}% reduction)",
            uncompressed_size,
            compressed.len(),
            (1.0 - compressed.len() as f64 / uncompressed_size as f64) * 100.0
        );
    };

    write_compressed("src/wkt2_definitions.bin.gz", &wkt_lines);
    write_compressed("src/projjson_definitions.bin.gz", &json_lines);

    println!("{count} PROJCRS entries generated, {skipped} files skipped");
}
