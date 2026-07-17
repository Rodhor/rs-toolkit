pub struct FileInfo {
    pub path: std::path::PathBuf,
    pub original_name: String,
    pub modified_time: std::time::SystemTime,
    pub accessed_time: std::time::SystemTime,
    pub created_time: std::time::SystemTime,
}
