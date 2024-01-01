use std::io;
use crate::{dirdata::DirData, Hashable};

// Holds data about a specific file we may be trying to find
pub struct FindFile {
    pub data: DirData,
    pub hash: String,
}

impl FindFile {
    pub fn new(path: String) -> Result<FindFile, io::Error> {
        let hash = path.get_file_hash()?;
        let data: DirData = DirData::new_from_path(path)?;
        
        Ok(FindFile{
            hash: hash,
            data: data,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_create_find_file() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let path_string: String = path.display().to_string();

        let find_file = FindFile::new(path_string);
        assert!(find_file.is_ok(), "no io error expected");

        if let Ok(find_file) = find_file {
            assert_eq!(find_file.hash, String::from("1577245F909F3D4619DDA56A7B4BA1AF"));
            assert_eq!(find_file.data.size, 100);
            assert_eq!(find_file.data.path, path);
        };
    }

    #[test]
    fn test_create_find_file_error() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","doesnotexist.txt"].iter().collect();
        let path_string: String = path.display().to_string();

        let find_file = FindFile::new(path_string);
        assert!(find_file.is_err(), "io error expected");
    }

}