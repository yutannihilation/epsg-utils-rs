//! Shared infrastructure for chunked gzip-compressed EPSG definitions.
//!
//! # Binary format
//!
//! Each `.bin` file has the following layout (all integers little-endian):
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │ Header (8 bytes)                                    │
//! │   u32  chunk_count                                  │
//! │   u32  entry_count                                  │
//! ├─────────────────────────────────────────────────────┤
//! │ Chunk table (chunk_count × 8 bytes)                 │
//! │   u32  compressed_offset  (into data section)       │
//! │   u32  compressed_len                               │
//! ├─────────────────────────────────────────────────────┤
//! │ Entry index (entry_count × 16 bytes, sorted by code)│
//! │   i32  epsg_code                                    │
//! │   u32  chunk_id                                     │
//! │   u32  offset_in_chunk  (byte offset after decomp.) │
//! │   u32  len              (byte length of the string) │
//! ├─────────────────────────────────────────────────────┤
//! │ Data section                                        │
//! │   Concatenated gzip streams, one per chunk.         │
//! │   Each stream decompresses to the concatenation of  │
//! │   all entries in that chunk (no separators).        │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! Entries within the index are sorted by EPSG code so that lookup can use
//! binary search. All entries in chunk *N* appear consecutively in the index,
//! and chunk IDs are assigned in code order so that nearby codes share a chunk.
//!
//! # Runtime behaviour
//!
//! On lookup, the index is binary-searched (no heap allocation). The matching
//! chunk is decompressed on demand and cached via `Box::leak` so that a
//! `&'static str` can be returned. In practice only a handful of chunks are
//! ever touched.

use std::io::Read as _;
use std::sync::Mutex;

use flate2::read::GzDecoder;

/// A lazily-decompressed, chunked definition store.
///
/// Constructed once per definition type (WKT2 / PROJJSON) and stored in a
/// `LazyLock`. The raw bytes come from `include_bytes!`.
pub(crate) struct ChunkedStore {
    /// The full `.bin` file contents (embedded at compile time).
    data: &'static [u8],
    /// Number of chunks (used only for sizing the cache vector).
    #[allow(dead_code)]
    chunk_count: u32,
    /// Number of entries across all chunks.
    entry_count: u32,
    /// Byte offset where the chunk table starts (always 8).
    chunk_table_offset: usize,
    /// Byte offset where the entry index starts.
    entry_index_offset: usize,
    /// Byte offset where the compressed data section starts.
    data_section_offset: usize,
    /// Per-chunk cache of decompressed text. Each slot starts as `None` and is
    /// filled on first access with a leaked `&'static str`.
    cache: Mutex<Vec<Option<&'static str>>>,
}

impl ChunkedStore {
    /// Parse the header of a `.bin` file and prepare the store for lookups.
    ///
    /// # Panics
    ///
    /// Panics if the data is too short to contain a valid header.
    pub(crate) fn new(data: &'static [u8]) -> Self {
        assert!(data.len() >= 8, "chunked definitions file too short");

        let chunk_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let entry_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        let chunk_table_offset = 8;
        let entry_index_offset = chunk_table_offset + (chunk_count as usize) * 8;
        let data_section_offset = entry_index_offset + (entry_count as usize) * 16;

        assert!(
            data.len() >= data_section_offset,
            "chunked definitions file truncated"
        );

        let cache = Mutex::new(vec![None; chunk_count as usize]);

        Self {
            data,
            chunk_count,
            entry_count,
            chunk_table_offset,
            entry_index_offset,
            data_section_offset,
            cache,
        }
    }

    /// Look up a definition string by EPSG code.
    ///
    /// Returns `None` if the code is not in the index. Decompresses the
    /// containing chunk on first access (subsequent lookups in the same chunk
    /// are free).
    pub(crate) fn lookup(&self, code: i32) -> Option<&'static str> {
        // Binary search the entry index for `code`.
        let n = self.entry_count as usize;
        let mut lo = 0usize;
        let mut hi = n;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let entry = self.read_entry(mid);
            match entry.code.cmp(&code) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => {
                    let chunk_str = self.ensure_chunk(entry.chunk_id as usize);
                    let start = entry.offset_in_chunk as usize;
                    let end = start + entry.len as usize;
                    return Some(&chunk_str[start..end]);
                }
            }
        }
        None
    }

    /// Read the `i`-th entry from the entry index.
    fn read_entry(&self, i: usize) -> IndexEntry {
        let base = self.entry_index_offset + i * 16;
        let d = self.data;
        IndexEntry {
            code: i32::from_le_bytes([d[base], d[base + 1], d[base + 2], d[base + 3]]),
            chunk_id: u32::from_le_bytes([d[base + 4], d[base + 5], d[base + 6], d[base + 7]]),
            offset_in_chunk: u32::from_le_bytes([
                d[base + 8],
                d[base + 9],
                d[base + 10],
                d[base + 11],
            ]),
            len: u32::from_le_bytes([d[base + 12], d[base + 13], d[base + 14], d[base + 15]]),
        }
    }

    /// Ensure chunk `chunk_id` is decompressed and cached, returning a
    /// `&'static str` view of the decompressed text.
    fn ensure_chunk(&self, chunk_id: usize) -> &'static str {
        // Fast path: already cached.
        {
            let cache = self.cache.lock().unwrap();
            if let Some(s) = cache[chunk_id] {
                return s;
            }
        }

        // Slow path: decompress the chunk.
        let (offset, len) = self.read_chunk_table(chunk_id);
        let compressed = &self.data[offset..offset + len];

        let mut decoder = GzDecoder::new(compressed);
        let mut text = String::new();
        decoder
            .read_to_string(&mut text)
            .expect("failed to decompress chunk");

        // Leak the string so we can return &'static str.
        let leaked: &'static str = Box::leak(text.into_boxed_str());

        let mut cache = self.cache.lock().unwrap();
        // Another thread may have filled it while we were decompressing.
        if let Some(s) = cache[chunk_id] {
            // We already leaked `text` — that's a small, bounded waste
            // (at most one chunk's worth per race).
            return s;
        }
        cache[chunk_id] = Some(leaked);
        leaked
    }

    /// Read the compressed offset and length for a chunk from the chunk table.
    fn read_chunk_table(&self, chunk_id: usize) -> (usize, usize) {
        let base = self.chunk_table_offset + chunk_id * 8;
        let d = self.data;
        let offset = u32::from_le_bytes([d[base], d[base + 1], d[base + 2], d[base + 3]]) as usize;
        let len = u32::from_le_bytes([d[base + 4], d[base + 5], d[base + 6], d[base + 7]]) as usize;
        // Offset in the chunk table is relative to the data section.
        (self.data_section_offset + offset, len)
    }
}

/// A single entry in the binary search index.
struct IndexEntry {
    code: i32,
    chunk_id: u32,
    offset_in_chunk: u32,
    len: u32,
}
