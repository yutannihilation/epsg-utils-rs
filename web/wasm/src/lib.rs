use wasm_bindgen::prelude::*;

/// Look up the WKT2 representation of an EPSG code.
#[wasm_bindgen]
pub fn epsg_to_wkt2(code: i32) -> Result<String, JsValue> {
    epsg_utils::epsg_to_wkt2(code)
        .map(|s| s.to_string())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Look up the PROJJSON representation of an EPSG code.
#[wasm_bindgen]
pub fn epsg_to_projjson(code: i32) -> Result<String, JsValue> {
    epsg_utils::epsg_to_projjson(code)
        .map(|s| s.to_string())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Parse a WKT2 string and return the PROJJSON representation.
#[wasm_bindgen]
pub fn wkt2_to_projjson(input: &str) -> Result<String, JsValue> {
    let crs = epsg_utils::parse_wkt2(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let value = crs.to_projjson();
    serde_json::to_string_pretty(&value).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Parse a WKT2 string and return the detected EPSG code (if any).
#[wasm_bindgen]
pub fn wkt2_to_epsg(input: &str) -> Result<JsValue, JsValue> {
    let crs = epsg_utils::parse_wkt2(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    match crs.to_epsg() {
        Some(code) => Ok(JsValue::from(code)),
        None => Ok(JsValue::NULL),
    }
}
