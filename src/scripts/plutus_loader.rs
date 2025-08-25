use serde_json::Value;
use std::fs;
use whisky::WError;

pub fn get_compiled_code_by_index(index: usize) -> Result<String, WError> {
    let json_content = fs::read_to_string("src/scripts/plutus.json")
        .map_err(|e| WError::new(&format!("Failed to read plutus.json: {}", e), "FileError"))?;

    let json: Value = serde_json::from_str(&json_content)
        .map_err(|e| WError::new(&format!("Failed to parse plutus.json: {}", e), "JsonError"))?;

    let validators_array = json["validators"]
        .as_array()
        .ok_or_else(|| WError::new("No validators array found in plutus.json", "JsonError"))?;

    if index >= validators_array.len() {
        return Err(WError::new(
            &format!("Index {} out of bounds for validators array", index),
            "IndexError",
        ));
    }

    let validator = &validators_array[index];
    let compiled_code = validator["compiledCode"].as_str().ok_or_else(|| {
        WError::new(
            &format!("No compiledCode found at index {}", index),
            "JsonError",
        )
    })?;

    Ok(compiled_code.to_string())
}
