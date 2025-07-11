use std::fs;
use axum::Json;
use serde_json::Value;


pub async fn get_idl() -> Result<Json<Value>, String> {
    let path = "programs/escrow/target/idl/escrow.json";
    println!("Attempting to read IDL from: {}", path);

    fs::read_to_string(path)
        .map_err(|e| format!("File read error: {}", e))
        .and_then(|data| {
            println!("Successfully read IDL ({} bytes)", data.len());
            println!("Sample data: {}", &data[..data.len().min(200)]);
            
            serde_json::from_str(&data)
                .map(Json)
                .map_err(|e| format!("JSON parse error: {}", e))
        })
}