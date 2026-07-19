use chrono::Local;
use glob::{Pattern, glob};
use std::{fs, time::SystemTime};

use crate::{
    config::{Settings, SortField},
    models::FileInfo,
};

pub fn list_files(settings: &Settings) -> Vec<FileInfo> {
    let exclude = if settings.exclude_pattern.is_empty() {
        None
    } else {
        match Pattern::new(&settings.exclude_pattern) {
            Ok(p) => Some(p),
            Err(e) => {
                tracing::error!(
                    "Invalid exclude pattern '{}': {}",
                    settings.exclude_pattern,
                    e
                );
                None
            }
        }
    };

    let pattern = settings.path.join(match settings.include_pattern.as_str() {
        "" => "*",
        s => s,
    });

    match glob(&pattern.to_string_lossy()) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries {
                let Ok(file_path) = entry else {
                    continue;
                };
                if file_path.is_file() {
                    let file_name = file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default();
                    if let Some(ref pattern) = exclude
                        && pattern.matches(file_name)
                    {
                        tracing::info!("Excluded file {} skipped", file_path.display());
                        continue;
                    }
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

            match settings.sort_field {
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

pub fn process_files(files: Vec<FileInfo>, settings: &Settings) {
    let archive_dir = if !settings.overwrite_original {
        let timestamp = Local::now().format("%Y%m%d_%H%M").to_string();
        let dir = settings.path.join(&timestamp);
        if let Err(e) = fs::create_dir_all(&dir) {
            tracing::error!("Failed to create archive directory: {}", e);
            return;
        }
        tracing::info!("Created archive directory at: {}", dir.display());
        Some(dir)
    } else {
        None
    };
    let mut current_value = 1;
    for file in files {
        let current_name = &file.original_name;
        if !current_name.is_empty() {
            let new_name = format!(
                "{num:0w$}_{name}",
                num = current_value,
                w = settings.width,
                name = current_name
            );

            if let Some(ref dir) = archive_dir {
                let archive_filepath = dir.join(current_name);
                if let Err(e) = fs::copy(&file.path, &archive_filepath) {
                    tracing::error!(
                        "Failed to copy file {} to archive: {}",
                        file.path.display(),
                        e
                    );
                    continue;
                } else {
                    tracing::info!(
                        "Copied file {} to archive: {}",
                        file.path.display(),
                        archive_filepath.display()
                    );
                }
            }

            let new_filepath = file.path.with_file_name(&new_name);
            match fs::rename(&file.path, &new_filepath) {
                Ok(_) => tracing::info!(
                    "Renamed file {} to {}",
                    file.path.display(),
                    new_filepath.display()
                ),
                Err(err) => tracing::error!("Error renaming file {}: {}", file.path.display(), err),
            };
            current_value += settings.gab;
        }
    }
}
