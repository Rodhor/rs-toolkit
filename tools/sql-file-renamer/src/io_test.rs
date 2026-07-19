use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::config::Settings;

/// Create an empty file called `name` inside `dir`.
fn touch(dir: &Path, name: &str) {
    fs::write(dir.join(name), b"").expect("failed to create test file");
}

/// Settings pointing at `dir`, archiving on (overwrite_original = false),
/// with an include pattern that matches every file by default.
fn settings_for(dir: &Path) -> Settings {
    Settings {
        path: dir.to_path_buf(),
        include_pattern: String::from("*"),
        ..Settings::default()
    }
}

/// The immediate sub-directories of `dir` (used to find the archive folder,
/// whose name is a non-deterministic timestamp).
fn subdirs(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .expect("failed to read dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect()
}

/// The file names directly inside `dir`, sorted for stable assertions.
fn file_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("failed to read dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn list_files_includes_matching_and_ignores_others() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "a.sql");
    touch(tmp.path(), "b.sql");
    touch(tmp.path(), "notes.txt");

    let mut settings = settings_for(tmp.path());
    settings.include_pattern = String::from("*.sql");

    let files = list_files(&settings);
    let mut names: Vec<&str> = files.iter().map(|f| f.original_name.as_str()).collect();
    names.sort();

    assert_eq!(names, vec!["a.sql", "b.sql"]);
}

#[test]
fn list_files_applies_exclude_pattern() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "00001_a.sql");
    touch(tmp.path(), "b.sql");

    let mut settings = settings_for(tmp.path());
    settings.include_pattern = String::from("*.sql");
    settings.exclude_pattern = String::from("00*");

    let files = list_files(&settings);
    let names: Vec<&str> = files.iter().map(|f| f.original_name.as_str()).collect();

    assert_eq!(names, vec!["b.sql"], "already-numbered files should be excluded");
}

#[test]
fn list_files_sorts_alphabetically() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "c.sql");
    touch(tmp.path(), "a.sql");
    touch(tmp.path(), "b.sql");

    let mut settings = settings_for(tmp.path());
    settings.include_pattern = String::from("*.sql");

    settings.sort_field = SortField::AlphabeticalAsc;
    let asc: Vec<String> = list_files(&settings)
        .into_iter()
        .map(|f| f.original_name)
        .collect();
    assert_eq!(asc, vec!["a.sql", "b.sql", "c.sql"]);

    settings.sort_field = SortField::AlphabeticalDesc;
    let desc: Vec<String> = list_files(&settings)
        .into_iter()
        .map(|f| f.original_name)
        .collect();
    assert_eq!(desc, vec!["c.sql", "b.sql", "a.sql"]);
}

#[test]
fn list_files_empty_pattern_matches_nothing() {
    // Characterizes the current behavior: an empty include_pattern globs the
    // directory itself, which is not a file, so nothing is returned.
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "a.sql");

    let mut settings = settings_for(tmp.path());
    settings.include_pattern = String::new();

    assert!(list_files(&settings).is_empty());
}

#[test]
fn process_files_archives_and_renames() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "a.sql");
    touch(tmp.path(), "b.sql");

    let mut settings = settings_for(tmp.path());
    settings.include_pattern = String::from("*.sql");
    settings.overwrite_original = false;
    settings.width = 5;
    settings.gab = 20;

    let files = list_files(&settings);
    process_files(files, &settings);

    // Source files are renamed in place with padded number + gap.
    assert_eq!(
        file_names(tmp.path()),
        vec!["00001_a.sql", "00021_b.sql"],
        "sources should be renamed in place"
    );

    // Exactly one archive directory, holding copies with the ORIGINAL names.
    let dirs = subdirs(tmp.path());
    assert_eq!(dirs.len(), 1, "expected a single archive directory");
    assert_eq!(
        file_names(&dirs[0]),
        vec!["a.sql", "b.sql"],
        "archive should keep the original file names"
    );
}

#[test]
fn process_files_overwrite_skips_archive() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "a.sql");

    let mut settings = settings_for(tmp.path());
    settings.include_pattern = String::from("*.sql");
    settings.overwrite_original = true;
    settings.width = 5;
    settings.gab = 20;

    let files = list_files(&settings);
    process_files(files, &settings);

    assert_eq!(file_names(tmp.path()), vec!["00001_a.sql"]);
    assert!(
        subdirs(tmp.path()).is_empty(),
        "no archive directory should be created when overwriting"
    );
}
