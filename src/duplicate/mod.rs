/// Holds information about a specific set of duplicate files
pub struct Duplicate {
    /// File contents hash that match occurred on
    pub hash: String,
    /// List of all full file paths that share the hash
    pub files: Vec<String>,
    /// Size of the files in bytes
    pub size: u64,
}

impl Clone for Duplicate {
    fn clone(&self) -> Duplicate {
        let hash = self.hash.clone();
        let files = self.files.clone();
        let size = self.size.clone();

        Duplicate { 
            hash: hash, 
            files: files, 
            size: size,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_clone_same() {
        let original: Duplicate = Duplicate{hash: String::from("12345"), files: vec![String::from("first"), String::from("second")], size: 542};
        let duplicate = original.clone();

        assert_eq!(original.hash, duplicate.hash);
        assert_eq!(original.size, duplicate.size);
        assert_eq!(original.files, duplicate.files);
    }

    #[test]
    fn test_clone_changed() {
        let original: Duplicate = Duplicate{hash: String::from("12345"), files: vec![String::from("first"), String::from("second")], size: 542};
        let mut duplicate = original.clone();

        assert_eq!(original.hash, duplicate.hash);
        assert_eq!(original.size, duplicate.size);
        assert_eq!(original.files, duplicate.files);

        duplicate.size = 539;
        duplicate.hash = String::from("54321");
        duplicate.files.remove(1);

        assert_ne!(original.hash, duplicate.hash);
        assert_ne!(original.size, duplicate.size);
        assert_ne!(original.files, duplicate.files);
        assert_eq!(original.files.len(), 2);
        assert_eq!(duplicate.files.len(), 1);

    }
}