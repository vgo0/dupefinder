use std::{path::PathBuf, fs::{Metadata, DirEntry, self}, io};

// convenience struct for holding unwrapped data
#[derive(Clone)]
pub struct DirData {
    pub path: PathBuf,
    pub meta: Metadata,
    pub size: u64,
}

impl DirData {
    pub fn new(path: Result<DirEntry, std::io::Error>) -> Result<DirData, Box<dyn std::error::Error>> {
        let path_data = path?;
        let meta_data = path_data.metadata()?;
        let size = meta_data.len();
        
        Ok(DirData{path: path_data.path(), meta: meta_data, size: size})
    }

    pub fn new_from_path(path: String) -> Result<DirData, io::Error> {
        let path_buf: PathBuf = path.clone().into();
        let meta_data = fs::metadata(path)?;
        let size = meta_data.len();

        Ok(DirData { path: path_buf, meta: meta_data, size: size })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from_direntry() {
        let directory: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let paths = fs::read_dir(directory).unwrap();

        let mut file_count = 0;
        for path in paths {
            let data = DirData::new(path);

            assert!(data.is_ok(), "no error should occur");
            if let Ok(data) = data {
                assert_eq!(data.size, 100);
                assert!(data.meta.is_file(), "entry should be a file");
            };

            file_count += 1;
        }

        assert_eq!(file_count, 2);
    }

    #[test]
    fn test_from_path_fail() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","doesnotexist.txt"].iter().collect();
        let path_string: String = path.display().to_string();
        let data = DirData::new_from_path(path_string);
        assert!(data.is_err(), "io error should occur");
    }

    #[test]
    fn test_from_path_file() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let path_string: String = path.display().to_string();
        let data = DirData::new_from_path(path_string);
        assert!(data.is_ok(), "no io error should occur");
        if let Ok(data) = data {
            assert_eq!(data.path, path);
            assert_eq!(data.size, 100);
            assert!(data.meta.is_file(), "entry should be a file");
        };
    }

    #[test]
    fn test_from_path_folder() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let path_string: String = path.display().to_string();
        let data = DirData::new_from_path(path_string);
        assert!(data.is_ok(), "no io error should occur");
        if let Ok(data) = data {
            assert_eq!(data.path, path);
            assert!(data.meta.is_dir(), "entry should be a folder");
        };
    }
}