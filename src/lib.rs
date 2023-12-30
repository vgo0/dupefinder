//! # Dupe Finder
//!
//! `dupefinder` is a utility for finding duplicate files within
//! a set of folders. The contents of each folder are evaluated against
//! all other provided folders. This means if file 'a.jpg' in folder 'one'
//! also exists as 'b.jpg' in folder 'two' that will be considered a match.
//! 
//! This utility works by parsing file metadata within the provided
//! folders and grouping together all files with the same size in bytes.
//! Once sizes with multiple file entries are located, the file contents are 
//! hashed via Sha256 and compared to the hash of other same-size files.
//! 
//! If only a single file of a certain size is found that file is not read and is skipped.
//! This does read the entire file contents from disk while generating the hash.
//! 
//! Hashing makes use of the `sha2` crate's compatibility with `Read`able object
//! which should prevent having to read the entirety of a file into memory at once to generate the hash.
//! 
//! If a matching hash is found, a duplicate file has been found and will be returned.
//! 
//! Matching can be run more than once on a single `DupeChecker` via `.run()`, this is a full re-check
//! of all folders with the assumption file contents may have changed not just the presence of files.
//! 
//! Matching will actively skip (continue) past problems. Warnings are emitted via the `log` crate
//! when such problems arise but they are otherwise not reported. Due to the support for multiple directories
//! and large file quantities stopping on a specific error was not desired.
//! 
//! There is an additional `.run_for_file()` mode that will only search for duplicates of a specific file.
//! 
//! # Examples
//! ```
//! let directories = vec![String::from("./")];
//! // non-recursive
//! let mut checker = dupefinder::DupeFinder::new(directories.clone());
//! let results = checker.run();
//! //recursive
//! let mut checker_recursive = dupefinder::DupeFinder::new_recursive(directories);
//! let results_recursive = checker_recursive.run();
//! // find if specific file has duplicates
//! let results = checker.run_for_file(String::from("./test.txt"));
//! ```

use std::{collections::{HashMap, HashSet}, fs, io};
use dirdata::DirData;
use findfile::FindFile;
use log::warn;
pub use hashable::Hashable;
pub use duplicate::Duplicate;

mod hashable;
mod dirdata;
mod duplicate;
mod findfile;

/// Searches for duplicate files in the provided directories / subdirectories
///
/// # Examples
/// ## Non-recursive
/// ```
/// let directories = vec![String::from("./resources")];
/// let mut checker = dupefinder::DupeFinder::new(directories);
/// let results = checker.run();
/// for key in results.keys() {
///     let result = results.get(key);
///     if let Some(details) = result {
///         println!("{} files of size {} bytes found with hash {}", details.files.len(), details.size, details.hash);
///         for file in details.files.iter() {
///             println!("{}", file);
///         }
///     }
/// }
/// ```
/// ## Recursive subfolder search
/// ```
/// let directories = vec![String::from("./resources")];
/// let mut checker = dupefinder::DupeFinder::new_recursive(directories);
/// let results = checker.run();
/// 
/// for key in results.keys() {
///     let result = results.get(key);
///     if let Some(details) = result {
///         println!("{} files of size {} bytes found with hash {}", details.files.len(), details.size, details.hash);
///         for file in details.files.iter() {
///             println!("{}", file);
///         }
///     }
/// }
/// ```
/// ## Specific file search
/// ```
/// let directories = vec![String::from("./resources")];
/// let mut checker = dupefinder::DupeFinder::new(directories);
/// let results = checker.run_for_file(String::from("./test.txt"));
/// 
/// if let Ok(results) = results {
///     match results {
///        Some(duplicate) => {
///            println!("{} files found", duplicate.files.len());
///         },
///        None => {
///            println!("no matching files found");
///        },
///     }
/// };
/// ```
pub struct DupeFinder {
    directories: Vec<String>,
    file_sizes: HashMap<u64, Vec<DirData>>,
    checked_directories: HashSet<String>,
    duplicate_file_sizes: HashSet<u64>,
    follow_subdirs: bool,
    find_file: Option<FindFile>,
}

impl DupeFinder {
    /// Initializes DupeFinder and provides the runnable checker
    pub fn new(directories: Vec<String>) -> DupeFinder {
        DupeFinder {
            directories: directories,
            file_sizes: HashMap::new(),
            checked_directories: HashSet::<String>::new(),
            duplicate_file_sizes: HashSet::new(),
            follow_subdirs: false,
            find_file: None,
        }
    }

    /// Initializes DupeFinder set to recursively traverse all subdirectories
    pub fn new_recursive(directories: Vec<String>) -> DupeFinder {
        DupeFinder {
            directories: directories,
            file_sizes: HashMap::new(),
            checked_directories: HashSet::<String>::new(),
            duplicate_file_sizes: HashSet::new(),
            follow_subdirs: true,
            find_file: None,
        }
    }

    // iterates through user provided directories and subdirectories
    // to build `file_sizes` map and mark entries with multiple sizes
    fn build_directories(&mut self) {
        let mut check_dirs = self.directories.clone();
        
        while check_dirs.len() > 0 {
            let mut next_directories: Vec<String> = Vec::new();

            for directory in check_dirs {
                if self.checked_directories.contains(&directory) {
                    continue
                }
    
                self.checked_directories.insert(directory.to_string());
    
                match self.build_directory_contents(&directory) {
                    Ok(mut next) =>  {
                        next_directories.append(&mut next);
                    },
                    Err(e) => {
                        warn!("An error building directory contents: {};", e);
                        continue;
                    }
                }
            }
            
            check_dirs = next_directories;
        }
    }

    fn check_path_duplicates(&self, paths: &Vec<DirData>, results: &mut HashMap<String, Duplicate>,) {
        // holds Hash -> Path values, if a hash is re-inserted here we know it is a dupe
        let mut known_hashes: HashMap<String, String> = HashMap::new();

        // entry @ 0 of paths in a find_file situation will be the original file
        // we will skip it and insert our known hash to avoid re-reading the file
        let iterator = match &self.find_file {
            Some(find_file) => {
                known_hashes.insert(find_file.hash.clone(), find_file.data.path.display().to_string());
                paths.iter().skip(1)
            },
            None => paths.iter().skip(0)
        };

        for data in iterator {
            let full_path = data.path.display().to_string();

            let file_hash: String = match data.path.get_file_hash() {
                Ok(hash) => hash,
                Err(e) => {
                    warn!("Error generating file hash for file: {}; error: {}", full_path, e);
                    continue;
                }
            };

            // if the hash already exists we will get a Some() value with the old entry
            let exists = known_hashes.insert(file_hash.clone(), full_path.clone());

            if let Some(existing_file) = exists {
                if results.contains_key(&file_hash) {
                    results.entry(file_hash).and_modify(|entry| entry.files.push(full_path.clone()));
                } else {
                    results.insert(file_hash.clone(), Duplicate { 
                        hash: file_hash, 
                        files: vec![existing_file.clone(), full_path.clone()], 
                        size: data.meta.len()
                    });
                }
            }
        }
    }

    // iterates through known sizes with multiple entries (`duplicate_file_sizes`)
    // and checks for dupes
    fn check_duplicates(&mut self, results: &mut HashMap<String, Duplicate>,) {
        for key in self.duplicate_file_sizes.iter() {
            let paths_o = self.file_sizes.get(key);
            if let Some(paths) = paths_o {
                self.check_path_duplicates(paths, results);
            } else {
                warn!("Error getting path data for key: {};", key);
                continue;
            }
        }
    }

    // If this object has already been .run() we need to reset 
    // our information. The assumption is that the contents of the files
    // may have changed between .run()'s, not just the presence of files
    // so we perform a full search again
    fn initialize(&mut self) {
        if self.checked_directories.len() > 0 {
            self.file_sizes = HashMap::new();
            self.checked_directories = HashSet::new();
            self.duplicate_file_sizes = HashSet::new();
            self.find_file = None;
        }
    }

    // inserts our original file into `file_sizes` which will trigger insertion
    // into `duplicate_file_sizes` if any other files with the same size exist
    fn insert_find_file_size(&mut self) {
        if let Some(find_file) = &self.find_file {
            self.file_sizes.insert(find_file.data.size, vec![find_file.data.clone()]);
        };
    }

    /// Runs the search to find if any duplicates of a specific file exist
    /// The resulting `Duplicate` will contain the original file if duplicates exist
    pub fn run_for_file(&mut self, path: String) -> Result<Option<Duplicate>, io::Error> {
        self.initialize();
        self.find_file = Some(FindFile::new(path)?);
        self.insert_find_file_size();

        self.build_directories();

        // dupes will be added to this map and returned
        let mut dupes: HashMap<String, Duplicate> = HashMap::new();
        self.check_duplicates(&mut dupes);

        if let Some(find_file) = &self.find_file {
            let result = dupes.get(&find_file.hash);
        
            match result {
                Some(value) => {
                    return Ok(Some(value.to_owned()));
                },
                None => {
                    return Ok(None);
                },
            }
        };

        Ok(None)
    }

    /// Runs the search for duplicate files and returns the matches
    pub fn run(&mut self) -> HashMap<String, Duplicate> {
        self.initialize();

        self.build_directories();

        // dupes will be added to this map and returned
        let mut dupes: HashMap<String, Duplicate> = HashMap::new();
        self.check_duplicates(&mut dupes);

        dupes
    }

    fn should_insert_size(&self, data: &DirData, subdirs: &mut Vec<String>) -> bool {
        if !data.meta.is_file() {
            if self.follow_subdirs && data.meta.is_dir() {
                subdirs.push(data.path.display().to_string());
            }

            return false;
        }

        // skip empty files
        if data.size == 0 {
            return false;
        }

        // we are in find file mode
        if let Some(find_file) = &self.find_file {
            // we only care about things that are the same size as our search file
            if data.size != find_file.data.size {
                return false;
            }

            // we want to skip our search file if it lives in the search directories
            if data.path == find_file.data.path {
                return false;
            }
        }

        true
    }
    
    fn build_directory_contents(&mut self, directory: &String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let paths = fs::read_dir(directory)?;
        // holds any found subdirectories if recursive search turned on
        let mut subdirs: Vec<String> = Vec::new();

        for path in paths {
            let data = match DirData::new(path) {
                Ok(val) => val,
                Err(e) => {
                    warn!("An error getting path / metadata: {}; skipped.", e);
                    continue;
                }
            };

            if self.should_insert_size(&data, &mut subdirs) {
                self.insert_size(data);
            }
        }
    
        Ok(subdirs)
    }

    fn insert_size(&mut self, data: DirData) {
        let len = data.meta.len();
        if self.file_sizes.contains_key(&len) {
            self.file_sizes.entry(len).and_modify(|list| list.push(data));
            self.duplicate_file_sizes.insert(len);
        } else {
            self.file_sizes.insert(len, vec![data]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_skips_folders() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "folders"].iter().collect();
        let dirs = vec![path.display().to_string()];

        let mut checker = DupeFinder::new(dirs);
        assert_eq!(checker.file_sizes.len(), 0);
        assert_eq!(checker.directories.len(), 1);

        let results = checker.run();
        assert_eq!(results.len(), 0);
        assert_eq!(checker.checked_directories.len(), 1);
        let known_size = 44;
        assert_known_size(&checker, known_size, 1, 1, 0);
    }

    #[test]
    fn check_skips_nonexistant() {
        let path_a: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes_directories", "dir_a"].iter().collect();
        let path_noexist: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes_directories", "noexist"].iter().collect();
        let dirs = vec![path_a.display().to_string(), path_noexist.display().to_string()];

        let mut checker = DupeFinder::new(dirs);
        assert_eq!(checker.file_sizes.len(), 0);
        assert_eq!(checker.directories.len(), 2);

        let results = checker.run();
        assert_eq!(results.len(), 0);
        assert_eq!(checker.checked_directories.len(), 2);
        let known_size = 100;
        assert_known_size(&checker, known_size, 1, 1, 0);
    }

    #[test]
    fn check_directory_only_once() {
        let path_a: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes_directories", "dir_a"].iter().collect();
        let path_a_dupe: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes_directories", "dir_a"].iter().collect();
        let dirs = vec![path_a.display().to_string(), path_a_dupe.display().to_string()];

        let mut checker = DupeFinder::new(dirs);
        assert_eq!(checker.file_sizes.len(), 0);
        assert_eq!(checker.directories.len(), 2);

        let results = checker.run();
        assert_eq!(results.len(), 0);
        assert_eq!(checker.checked_directories.len(), 1);
        let known_size = 100;
        assert_known_size(&checker, known_size, 1, 1, 0);
    }

    #[test]
    fn duplicate_recursive_works() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes_directories"].iter().collect();
        let dirs = vec![path.display().to_string()];

        let mut checker = DupeFinder::new_recursive(dirs);
        assert_eq!(checker.file_sizes.len(), 0);
        assert_eq!(checker.directories.len(), 1);

        let results = checker.run();
        assert_eq!(results.len(), 1);
        assert_eq!(checker.checked_directories.len(), 3);
        let known_size: u64 = 100;
        assert_known_size(&checker, known_size, 2, 1, 1);


        // re-run and make sure we get same information
        checker.run();
        assert_known_size(&checker, known_size, 2, 1, 1);
    }

    #[test]
    fn duplicate_different_directory_works() {
        let path_a: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes_directories", "dir_a"].iter().collect();
        let path_b: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes_directories", "dir_b"].iter().collect();
        let dirs = vec![path_a.display().to_string(), path_b.display().to_string()];

        let mut checker = DupeFinder::new(dirs);
        assert_eq!(checker.file_sizes.len(), 0);
        assert_eq!(checker.directories.len(), 2);

        let results = checker.run();
        assert_eq!(results.len(), 1);
        assert_eq!(checker.checked_directories.len(), 2);
        let known_size: u64 = 100;
        assert_known_size(&checker, known_size, 2, 1, 1);

        // re-run and make sure we get same information
        checker.run();
        assert_known_size(&checker, known_size, 2, 1, 1);
    }

    #[test]
    fn findfile_error() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let ff_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "base", "doesnotexist.txt"].iter().collect();
        let mut checker = DupeFinder::new(vec![path.display().to_string()]);
        assert_eq!(checker.file_sizes.len(), 0);

        let result = checker.run_for_file(ff_path.display().to_string());
        
        assert!(result.is_err(), "io error expected");
    }

    #[test]
    fn findfile_works_same_directory() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let ff_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes", "a.txt"].iter().collect();
        let mut checker = DupeFinder::new(vec![path.display().to_string()]);
        assert_eq!(checker.file_sizes.len(), 0);

        let result = checker.run_for_file(ff_path.display().to_string());
        
        let known_size: u64 = 100;
        
        assert!(result.is_ok(), "no io error expected");

        let duplicate = result.unwrap();
        assert!(duplicate.is_some(), "expected match");

        if let Some(duplicate) = duplicate {
            assert_eq!(duplicate.size, known_size);
            assert_eq!(duplicate.hash, String::from("AE040FB6B2256BD5CEADF0CA34262BAB9460B46613C718F86A47D5F657BAEC78"));
            assert_eq!(duplicate.files.len(), 2);
            assert!(duplicate.files.contains(&ff_path.display().to_string()));
        };
    }

    #[test]
    fn findfile_works_no_dupes() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "base"].iter().collect();
        let ff_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "base", "a.txt"].iter().collect();
        let mut checker = DupeFinder::new(vec![path.display().to_string()]);
        assert_eq!(checker.file_sizes.len(), 0);

        let result = checker.run_for_file(ff_path.display().to_string());
        
        assert!(result.is_ok(), "no io error expected");

        let duplicate = result.unwrap();
        assert!(duplicate.is_none(), "did not expect match");
    }

    #[test]
    fn findfile_works_dupes() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let ff_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "base", "a.txt"].iter().collect();
        let mut checker = DupeFinder::new(vec![path.display().to_string()]);
        assert_eq!(checker.file_sizes.len(), 0);

        let result = checker.run_for_file(ff_path.display().to_string());
        let known_size: u64 = 100;
        
        assert!(result.is_ok(), "no io error expected");

        let duplicate = result.unwrap();
        assert!(duplicate.is_some(), "expected match");

        if let Some(duplicate) = duplicate {
            assert_eq!(duplicate.size, known_size);
            assert_eq!(duplicate.hash, String::from("AE040FB6B2256BD5CEADF0CA34262BAB9460B46613C718F86A47D5F657BAEC78"));
            assert_eq!(duplicate.files.len(), 3);
            assert!(duplicate.files.contains(&ff_path.display().to_string()));
        };
    }

    #[test]
    fn duplicate_same_directory_works() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let mut checker = DupeFinder::new(vec![path.display().to_string()]);

        assert_eq!(checker.file_sizes.len(), 0);
        let results = checker.run();
        let known_size: u64 = 100;
        assert_eq!(results.len(), 1);
        assert_known_size(&checker, known_size, 2, 1, 1);

        // re-run and make sure we get same information
        checker.run();
        assert_known_size(&checker, known_size, 2, 1, 1);
    }

    #[test]
    fn multiple_runs_works() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "insert_size"].iter().collect();
        let mut checker = DupeFinder::new(vec![path.display().to_string()]);

        assert_eq!(checker.file_sizes.len(), 0);
        checker.run();
        let known_size: u64 = 44;
        assert_known_size(&checker, known_size, 1, 1, 0);

        // re-run and make sure we get same information
        checker.run();
        assert_known_size(&checker, known_size, 1, 1, 0);
    }

    #[test]
    fn should_not_insert_folder_recurse() {
        let dir_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let checker = DupeFinder::new_recursive(vec![dir_path.display().to_string()]);

        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let path_string: String = path.display().to_string();
        let data = DirData::new_from_path(path_string);
        let mut subdirs: Vec<String> = Vec::new();
        if let Ok(data) = data {
            assert!(!checker.should_insert_size(&data, &mut subdirs), "folder should be inserted");
            assert_eq!(subdirs.len(), 1);
        };
    }

    #[test]
    fn should_not_insert_folder() {
        let dir_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let checker = DupeFinder::new(vec![dir_path.display().to_string()]);

        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let path_string: String = path.display().to_string();
        let data = DirData::new_from_path(path_string);
        let mut subdirs: Vec<String> = Vec::new();
        if let Ok(data) = data {
            assert!(!checker.should_insert_size(&data, &mut subdirs), "folder should be inserted");
            assert_eq!(subdirs.len(), 0);
        };
    }

    #[test]
    fn should_insert_file() {
        let dir_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let checker = DupeFinder::new(vec![dir_path.display().to_string()]);

        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let path_string: String = path.display().to_string();
        let data = DirData::new_from_path(path_string);
        let mut subdirs: Vec<String> = Vec::new();
        if let Ok(data) = data {
            assert!(checker.should_insert_size(&data, &mut subdirs), "file should be inserted");
        };
    }

    #[test]
    fn should_not_insert_empty_file() {
        let dir_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes_with_empty"].iter().collect();
        let checker = DupeFinder::new(vec![dir_path.display().to_string()]);

        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes_with_empty","empty.txt"].iter().collect();
        let path_string: String = path.display().to_string();
        let data = DirData::new_from_path(path_string);
        let mut subdirs: Vec<String> = Vec::new();
        if let Ok(data) = data {
            assert!(!checker.should_insert_size(&data, &mut subdirs), "file should not be inserted");
        };
    }

    #[test]
    fn should_not_insert_findfile_file_same_file() {
        let dir_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let mut checker = DupeFinder::new(vec![dir_path.display().to_string()]);

        let ff_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let ff_path_string: String = ff_path.display().to_string();
        checker.find_file = Some(FindFile::new(ff_path_string).unwrap());

        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let path_string: String = path.display().to_string();
        let data = DirData::new_from_path(path_string);
        let mut subdirs: Vec<String> = Vec::new();
        if let Ok(data) = data {
            assert!(!checker.should_insert_size(&data, &mut subdirs), "file should be inserted");
        };
    }

    #[test]
    fn should_not_insert_findfile_file_diff_size() {
        let dir_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let mut checker = DupeFinder::new(vec![dir_path.display().to_string()]);

        let ff_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "insert_size","test.txt"].iter().collect();
        let ff_path_string: String = ff_path.display().to_string();
        checker.find_file = Some(FindFile::new(ff_path_string).unwrap());

        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","b.txt"].iter().collect();
        let path_string: String = path.display().to_string();
        let data = DirData::new_from_path(path_string);
        let mut subdirs: Vec<String> = Vec::new();
        if let Ok(data) = data {
            assert!(!checker.should_insert_size(&data, &mut subdirs), "file should be inserted");
        };
    }

    #[test]
    fn should_insert_findfile_file() {
        let dir_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes"].iter().collect();
        let mut checker = DupeFinder::new(vec![dir_path.display().to_string()]);

        let ff_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let ff_path_string: String = ff_path.display().to_string();
        checker.find_file = Some(FindFile::new(ff_path_string).unwrap());

        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","b.txt"].iter().collect();
        let path_string: String = path.display().to_string();
        let data = DirData::new_from_path(path_string);
        let mut subdirs: Vec<String> = Vec::new();
        if let Ok(data) = data {
            assert!(checker.should_insert_size(&data, &mut subdirs), "file should be inserted");
        };
    }

    #[test]
    fn insert_size_works() {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "insert_size"].iter().collect();
        let mut checker = DupeFinder::new(vec![path.display().to_string()]);

        assert_eq!(checker.file_sizes.len(), 0);
        checker.run();

        let known_size: u64 = 44;
        assert_known_size(&checker, known_size, 1, 1, 0);
    }

    #[test]
    fn insert_find_file_size_works() {
        let dir_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "insert_size"].iter().collect();
        let mut checker = DupeFinder::new(vec![dir_path.display().to_string()]);
        assert_eq!(checker.file_sizes.len(), 0);

        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "dupes","a.txt"].iter().collect();
        let path_string: String = path.display().to_string();
        checker.find_file = Some(FindFile::new(path_string).unwrap());

        checker.insert_find_file_size();
        assert_eq!(checker.file_sizes.len(), 1);
        assert!(checker.file_sizes.contains_key(&100));
        let data = checker.file_sizes.get(&100);
        let dir_data = data.unwrap();
        assert_eq!(dir_data.len(), 1);
    }

    fn assert_known_size(checker: &DupeFinder, known_size: u64, expected_files_known: usize, expected_total_sizes: usize, expected_duplicate_sizes: usize) {
        assert_eq!(checker.file_sizes.len(), expected_total_sizes);
        assert_eq!(checker.duplicate_file_sizes.len(), expected_duplicate_sizes);
        assert!(checker.file_sizes.contains_key(&known_size));
        assert_eq!(checker.file_sizes[&known_size].len(), expected_files_known);
    }
}
