use std::fs;
use std::io::Write;
use std::path::Path;

use flate2::Compression;
use flate2::write::GzEncoder;

/// Number of entries per independently-compressed chunk.
///
/// Each chunk is a separate gzip stream so that a runtime lookup only needs to
/// decompress one chunk (≈64 entries) rather than the entire dataset. The value
/// is a trade-off: smaller chunks mean less data to decompress per lookup, but
/// worse compression ratios due to less cross-entry repetition.
const CHUNK_SIZE: usize = 64;

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

    // Parse all CRS files into (code, wkt, projjson) triples.
    let mut entries: Vec<(i32, String, String)> = Vec::new();
    let mut skipped = 0u32;

    for filename in &files {
        let path = wkt_dir.join(filename);
        let wkt = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
        let wkt = wkt.trim().to_string();

        let crs = match epsg_utils::parse_wkt2(&wkt) {
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

        entries.push((code, wkt, projjson));
    }

    // Sort by code so the entry index supports binary search at runtime.
    entries.sort_by_key(|(code, _, _)| *code);

    let count = entries.len();

    write_chunked_bin(
        "src/wkt2_definitions.bin",
        &entries
            .iter()
            .map(|(c, w, _)| (*c, w.as_str()))
            .collect::<Vec<_>>(),
    );
    write_chunked_bin(
        "src/projjson_definitions.bin",
        &entries
            .iter()
            .map(|(c, _, j)| (*c, j.as_str()))
            .collect::<Vec<_>>(),
    );

    println!("{count} CRS entries generated, {skipped} files skipped");
}

/// Write a chunked `.bin` file.
///
/// The binary format is documented in `src/chunked_definitions.rs`. In short:
///
/// ```text
/// [header]         8 bytes: u32 chunk_count, u32 entry_count
/// [chunk table]    chunk_count × 8 bytes: (u32 offset, u32 len) per chunk
/// [entry index]    entry_count × 12 bytes: (i32 code, u16 chunk_id,
///                                           u16 offset_in_chunk, u32 len)
/// [data section]   concatenated gzip streams
/// ```
fn write_chunked_bin(path: &str, entries: &[(i32, &str)]) {
    let chunks: Vec<&[(i32, &str)]> = entries.chunks(CHUNK_SIZE).collect();
    let chunk_count = chunks.len() as u32;
    let entry_count = entries.len() as u32;

    // --- Compress each chunk independently ---
    struct ChunkInfo {
        compressed: Vec<u8>,
    }
    let mut chunk_infos: Vec<ChunkInfo> = Vec::with_capacity(chunks.len());
    let mut total_uncompressed = 0usize;

    // For each chunk, also record (code, chunk_id, offset_in_chunk, len) per entry.
    struct EntryInfo {
        code: i32,
        chunk_id: u16,
        offset_in_chunk: u16,
        len: u32,
    }
    let mut entry_infos: Vec<EntryInfo> = Vec::with_capacity(entries.len());

    for (chunk_id, chunk) in chunks.iter().enumerate() {
        // Concatenate all entry strings in this chunk (no separator needed;
        // entries are located by offset + length).
        let mut blob = String::new();
        for &(code, text) in *chunk {
            let offset_in_chunk = blob.len();
            blob.push_str(text);
            entry_infos.push(EntryInfo {
                code,
                chunk_id: chunk_id as u16,
                offset_in_chunk: offset_in_chunk as u16,
                len: text.len() as u32,
            });
        }
        total_uncompressed += blob.len();

        // Gzip-compress the concatenated blob.
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(blob.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        chunk_infos.push(ChunkInfo { compressed });
    }

    // --- Assemble the binary file ---
    let mut out = Vec::new();

    // Header
    out.extend_from_slice(&chunk_count.to_le_bytes());
    out.extend_from_slice(&entry_count.to_le_bytes());

    // Chunk table: (u32 offset_in_data_section, u32 compressed_len)
    let mut data_offset = 0u32;
    for ci in &chunk_infos {
        out.extend_from_slice(&data_offset.to_le_bytes());
        out.extend_from_slice(&(ci.compressed.len() as u32).to_le_bytes());
        data_offset += ci.compressed.len() as u32;
    }

    // Entry index: (i32 code, u16 chunk_id, u16 offset_in_chunk, u32 len)
    for ei in &entry_infos {
        out.extend_from_slice(&ei.code.to_le_bytes());
        out.extend_from_slice(&ei.chunk_id.to_le_bytes());
        out.extend_from_slice(&ei.offset_in_chunk.to_le_bytes());
        out.extend_from_slice(&ei.len.to_le_bytes());
    }

    // Data section: concatenated gzip streams
    let data_section_start = out.len();
    for ci in &chunk_infos {
        out.extend_from_slice(&ci.compressed);
    }

    let total_compressed: usize = chunk_infos.iter().map(|c| c.compressed.len()).sum();

    fs::write(path, &out).unwrap();
    println!(
        "Generated {path} ({} entries in {} chunks, \
         uncompressed {total_uncompressed} -> file {} bytes, \
         data section {total_compressed} bytes, {:.0}% reduction)",
        entry_infos.len(),
        chunk_infos.len(),
        out.len(),
        (1.0 - total_compressed as f64 / total_uncompressed as f64) * 100.0
    );
    let _ = data_section_start; // suppress unused warning
}
