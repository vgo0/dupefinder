use std::{io, path::PathBuf};
use std::io::{BufRead, BufReader};
use xxhash_rust::xxh3::Xxh3;

/// Convenience trait to generate a XXH3 hash of the file contents
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
    let file = std::fs::File::open(path)?;
    let mut file = BufReader::with_capacity(262144 , file);

    let mut hasher = Xxh3::default();
    loop {
        let buf = file.fill_buf()?;
        let buf_len = buf.len();
        if buf_len == 0 {
            break;
        }
        hasher.update(buf);
        file.consume(buf_len);
    }

    Ok(format!("{:X}", hasher.digest128()))
}



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_generate_file_hash() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let hash = generate_file_hash(path);
        assert!(hash.is_ok(), "no io error should occur");
        assert_eq!(hash.unwrap(), String::from("1577245F909F3D4619DDA56A7B4BA1AF"));
    }

    #[test]
    fn test_str_path_hash() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let path_str = path.to_str();

        assert!(path_str.is_some(), "no io error should occur");
        if let Some(path_str_val) = path_str {
            let hash = path_str_val.get_file_hash();
            assert!(hash.is_ok(), "no io error should occur");
            assert_eq!(hash.unwrap(), String::from("1577245F909F3D4619DDA56A7B4BA1AF"));
        }
    }

    #[test]
    fn test_string_path_hash() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let path_string: String = path.display().to_string();

        let hash = path_string.get_file_hash();
        assert!(hash.is_ok(), "no io error should occur");
        assert_eq!(hash.unwrap(), String::from("1577245F909F3D4619DDA56A7B4BA1AF"));
    }

    #[test]
    fn test_pathbuf_hash() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();

        let hash = path.get_file_hash();
        assert!(hash.is_ok(), "no io error should occur");
        assert_eq!(hash.unwrap(), String::from("1577245F909F3D4619DDA56A7B4BA1AF"));
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