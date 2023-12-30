use std::{io, path::PathBuf};
use sha2::{Sha256, Digest};

/// Convenience trait to generate a Sha256 hash of the file contents
/// located in the path specified by a `String` / `&str` / `PathBuf`.
/// # Examples
/// ```
/// use dupefinder::Hashable;
/// 
/// let path: std::path::PathBuf = std::path::PathBuf::from("./test.txt");
/// if let Ok(hash) = path.get_file_hash() {
///     println!("The file hash is: {}", hash);
/// };
/// ```
/// 
/// ```
/// use dupefinder::Hashable;
/// 
/// let path: String = String::from("./test.txt");
/// if let Ok(hash) = path.get_file_hash() {
///     println!("The file hash is: {}", hash);
/// };
/// ```
/// 
/// ```
/// use dupefinder::Hashable;
/// 
/// let path = "./test.txt";
/// if let Ok(hash) = path.get_file_hash() {
///     println!("The file hash is: {}", hash);
/// };
/// ```
pub trait Hashable {
    fn get_file_hash(&self) -> Result<String, io::Error>;
}

impl Hashable for String {
    fn get_file_hash(&self) -> Result<String, io::Error> {
        let path: PathBuf = self.into();

        generate_file_hash(path)
    }
}

impl Hashable for PathBuf {
    fn get_file_hash(&self) -> Result<String, io::Error> {
        generate_file_hash(self.to_path_buf())
    }
}

impl Hashable for &str {
    fn get_file_hash(&self) -> Result<String, io::Error> {
        let path: PathBuf = self.into();
        
        generate_file_hash(path)
    }
}

fn generate_file_hash(path: PathBuf) -> Result<String, io::Error> {
    let mut file = match std::fs::File::open(path) {
        Ok(val) => val,
        Err(e) => {
            return Err(e);
        }
    };

    let mut hasher = Sha256::new();
    if let Err(e) = io::copy(&mut file, &mut hasher) {
        return Err(e);
    }

    Ok(format!("{:X}", hasher.finalize()))
}



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_generate_file_hash() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let hash = generate_file_hash(path);
        assert!(hash.is_ok(), "no io error should occur");
        assert_eq!(hash.unwrap(), String::from("AE040FB6B2256BD5CEADF0CA34262BAB9460B46613C718F86A47D5F657BAEC78"));
    }

    #[test]
    fn test_str_path_hash() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let path_str = path.to_str();

        assert!(path_str.is_some(), "no io error should occur");
        if let Some(path_str_val) = path_str {
            let hash = path_str_val.get_file_hash();
            assert!(hash.is_ok(), "no io error should occur");
            assert_eq!(hash.unwrap(), String::from("AE040FB6B2256BD5CEADF0CA34262BAB9460B46613C718F86A47D5F657BAEC78"));
        }
    }

    #[test]
    fn test_string_path_hash() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let path_string: String = path.display().to_string();

        let hash = path_string.get_file_hash();
        assert!(hash.is_ok(), "no io error should occur");
        assert_eq!(hash.unwrap(), String::from("AE040FB6B2256BD5CEADF0CA34262BAB9460B46613C718F86A47D5F657BAEC78"));
    }

    #[test]
    fn test_pathbuf_hash() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();

        let hash = path.get_file_hash();
        assert!(hash.is_ok(), "no io error should occur");
        assert_eq!(hash.unwrap(), String::from("AE040FB6B2256BD5CEADF0CA34262BAB9460B46613C718F86A47D5F657BAEC78"));
    }

    #[test]
    fn test_generate_file_hash_error() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","doesnotexist.txt"].iter().collect();
        let hash = generate_file_hash(path);
        assert!(hash.is_err(), "io error should occur");
    }

    #[test]
    fn test_str_path_hash_error() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","doesnotexist.txt"].iter().collect();
        let path_str = path.to_str();

        assert!(path_str.is_some(), "no io error should occur");
        if let Some(path_str_val) = path_str {
            let hash = path_str_val.get_file_hash();
            assert!(hash.is_err(), "io error should occur");
        }
    }

    #[test]
    fn test_string_path_hash_error() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","doesnotexist.txt"].iter().collect();
        let path_string: String = path.display().to_string();

        let hash = path_string.get_file_hash();
        assert!(hash.is_err(), "io error should occur");
    }

    #[test]
    fn test_pathbuf_hash_error() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","doesnotexist.txt"].iter().collect();

        let hash = path.get_file_hash();
        assert!(hash.is_err(), "io error should occur");
    }
}