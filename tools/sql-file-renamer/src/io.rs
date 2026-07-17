use glob::glob;
use std::{fs, path::PathBuf, time::SystemTime};

use crate::{config::SortField, models::FileInfo};

pub fn list_files(path: PathBuf, file_pattern: &str, sorting: SortField) -> Vec<FileInfo> {
    let full_pattern = path.join(file_pattern);
    let pattern_str = full_pattern.to_string_lossy();

    match glob(&pattern_str) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries {
                let Ok(file_path) = entry else {
                    continue;
                };
                if file_path.is_file() {
                    let Ok(meta) = fs::metadata(&file_path) else {
                        continue;
                    };
                    let file_info = FileInfo {
                        original_name: file_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or_default()
                            .to_string(),
                        modified_time: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                        accessed_time: meta.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
                        created_time: meta.created().unwrap_or(SystemTime::UNIX_EPOCH),
                        path: file_path,
                    };
                    files.push(file_info);
                }
            }

            match sorting {
                SortField::AlphabeticalAsc => {
                    files.sort_by(|a, b| natord::compare(&a.original_name, &b.original_name));
                }
                SortField::AlphabeticalDesc => {
                    files.sort_by(|a, b| natord::compare(&b.original_name, &a.original_name));
                }
                SortField::ModifiedTimeNewest => {
                    files.sort_by_key(|b| std::cmp::Reverse(b.modified_time));
                }
                SortField::ModifiedTimeOldest => {
                    files.sort_by_key(|f| f.modified_time);
                }
                SortField::CreatedTimeNewest => {
                    files.sort_by_key(|b| std::cmp::Reverse(b.created_time));
                }
                SortField::CreatedTimeOldest => {
                    files.sort_by_key(|f| f.created_time);
                }
                SortField::AccessedTimeNewest => {
                    files.sort_by_key(|b| std::cmp::Reverse(b.accessed_time));
                }
                SortField::AccessedTimeOldest => {
                    files.sort_by_key(|f| f.accessed_time);
                }
            }

            tracing::info!("Found {} files in directory", files.len());
            files
        }
        Err(err) => {
            tracing::error!("Error reading directory: {}", err);
            Vec::new()
        }
    }
}

pub fn rename_files(files: Vec<FileInfo>) {
    let width = 9;
    let gab = 50;
    let mut current_value = 1;
    for file in files {
        let current_name = &file.original_name;
        if !current_name.is_empty() {
            let new_name = format!(
                "{num:0w$}_{name}",
                num = current_value,
                w = width,
                name = current_name
            );

            let mut new_filepath = file.path.clone();
            new_filepath.set_file_name(new_name);

            match fs::rename(&file.path, &new_filepath) {
                Ok(_) => tracing::info!(
                    "Renamed file {} to {}",
                    file.path.display(),
                    new_filepath.display()
                ),
                Err(err) => tracing::error!("Error renaming file {}: {}", file.path.display(), err),
            };
            current_value += gab;
        }
    }
}
