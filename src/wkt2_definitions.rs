//! Auto-generated EPSG CRS definitions (WKT2).
//!
//! The EPSG Dataset is owned by the International Association of Oil & Gas
//! Producers (IOGP). The source definitions were downloaded from
//! <https://epsg.org/download-dataset.html>.
//!
//! Definitions are stored as gzip-compressed data and decompressed on first access.

use std::collections::HashMap;
use std::io::Read;
use std::sync::LazyLock;

use flate2::read::GzDecoder;

static DATA: LazyLock<HashMap<i32, String>> = LazyLock::new(|| {
    let compressed = include_bytes!("wkt2_definitions.bin.gz");
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut text = String::new();
    decoder
        .read_to_string(&mut text)
        .expect("failed to decompress WKT2 definitions");

    let mut map = HashMap::new();
    for line in text.lines() {
        if let Some((code_str, wkt)) = line.split_once('\t')
            && let Ok(code) = code_str.parse::<i32>()
        {
            map.insert(code, wkt.to_string());
        }
    }
    map
});

pub(crate) fn lookup(code: i32) -> Option<&'static str> {
    DATA.get(&code).map(String::as_str)
}
