use crate::models::Item;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

fn data_file() -> Option<PathBuf> {
    let project = ProjectDirs::from("com", "DataConsole", "DataConsole")?;
    let directory = project.data_local_dir();

    fs::create_dir_all(directory).ok()?;

    Some(directory.join("items.json"))
}

pub fn load_items() -> Result<Vec<Item>, String> {
    let Some(path) = data_file() else {
        return Err("Could not determine the application data directory.".into());
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Could not read saved data: {e}"))?;

    serde_json::from_str(&contents).map_err(|e| format!("Saved data is invalid: {e}"))
}

pub fn save_items(items: &[Item]) -> Result<(), String> {
    let path = data_file().ok_or("Could not determine the application data directory.")?;

    let contents = serde_json::to_string_pretty(items)
        .map_err(|e| format!("Could not serialize data: {e}"))?;

    let temporary_path = path.with_extension("json.tmp");

    fs::write(&temporary_path, contents).map_err(|e| format!("Could not write saved data: {e}"))?;

    fs::rename(&temporary_path, &path)
        .map_err(|e| format!("Could not finalize saved data: {e}"))?;

    Ok(())
}
