use crate::models::Item;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

fn data_file() -> Result<PathBuf, String> {
    let dirs = ProjectDirs::from("com", "Azhar", "RustCrudApp")
        .ok_or_else(|| "Could not locate the application data directory".to_string())?;

    let directory = dirs.data_local_dir();
    fs::create_dir_all(directory).map_err(|e| format!("Could not create data directory: {e}"))?;

    Ok(directory.join("items.json"))
}

pub fn storage_path_label() -> String {
    match data_file() {
        Ok(path) => path.display().to_string(),
        Err(error) => error,
    }
}

pub fn load_items() -> Result<Vec<Item>, String> {
    let path = data_file()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Could not read saved records: {e}"))?;

    serde_json::from_str(&contents).map_err(|e| format!("Could not parse saved records: {e}"))
}

pub fn save_items(items: &[Item]) -> Result<(), String> {
    let path = data_file()?;

    let contents = serde_json::to_string_pretty(items)
        .map_err(|e| format!("Could not serialize records: {e}"))?;

    let temporary_path = path.with_extension("json.tmp");

    fs::write(&temporary_path, contents).map_err(|e| format!("Could not write records: {e}"))?;

    fs::rename(&temporary_path, &path)
        .map_err(|e| format!("Could not finalize saved records: {e}"))?;

    Ok(())
}
