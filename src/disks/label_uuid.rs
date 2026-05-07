use std::{collections::HashMap, fs, path::Path};

pub fn device_labels() -> HashMap<String, String> {
    read_disk_by_dir("/dev/disk/by-label")
}

pub fn device_uuids() -> HashMap<String, String> {
    read_disk_by_dir("/dev/disk/by-uuid")
}

fn read_disk_by_dir(dir: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let path = Path::new(dir);

    if !path.exists() {
        return map;
    }

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return map,
    };

    for entry in entries.flatten() {
        if let Ok(target) = fs::canonicalize(entry.path())
            && let Some(name) = entry.file_name().to_str()
        {
            map.insert(target.to_string_lossy().into_owned(), name.to_string());
        }
    }

    map
}
