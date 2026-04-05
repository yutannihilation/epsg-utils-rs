//! Auto-generated EPSG CRS definitions (PROJJSON).
//!
//! The EPSG Dataset is owned by the International Association of Oil & Gas
//! Producers (IOGP). The source definitions were downloaded from
//! <https://epsg.org/download-dataset.html>.
//!
//! Definitions are stored as chunked gzip-compressed data. Each chunk contains
//! ~64 entries and is decompressed independently on first access. See
//! [`chunked_definitions`](crate::chunked_definitions) for the binary format.

use std::sync::LazyLock;

use crate::chunked_definitions::ChunkedStore;

static STORE: LazyLock<ChunkedStore> =
    LazyLock::new(|| ChunkedStore::new(include_bytes!("projjson_definitions.bin")));

pub(crate) fn lookup(code: i32) -> Option<&'static str> {
    STORE.lookup(code)
}
