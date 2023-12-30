[![Rust](https://github.com/vgo0/dupefinder/actions/workflows/rust.yml/badge.svg)](https://github.com/vgo0/dupefinder/actions/workflows/rust.yml)

# Dupe Finder
`dupefinder` is a utility for finding duplicate files within
a set of folders. The contents of each folder are evaluated against
all other provided folders. This means if file 'a.jpg' in folder 'one'
also exists as 'b.jpg' in folder 'two' that will be considered a match.

This utility works by parsing file metadata within the provided
folders and grouping together all files with the same size in bytes.
Once sizes with multiple file entries are located, the file contents are 
hashed via Sha256 and compared to the hash of other same-size files.

If only a single file of a certain size is found that file is not read and is skipped.
This does read the entire file contents from disk while generating the hash.

Hashing makes use of the `sha2` crate's compatibility with `Read`able object
which should prevent having to read the entirety of a file into memory at once to generate the hash.

If a matching hash is found, a duplicate file has been found and will be returned.

Matching can be run more than once on a single `DupeChecker` via `.run()`, this is a full re-check
of all folders with the assumption file contents may have changed not just the presence of files.

Matching will actively skip (continue) past problems. Warnings are emitted via the `log` crate
when such problems arise but they are otherwise not reported. Due to the support for multiple directories
and large file quantities stopping on a specific error was not desired.

There is an additional `.run_for_file()` mode that will only search for duplicates of a specific file.

# Examples
## Non-recursive
```
let directories = vec![String::from("./resources")];
let mut checker = dupefinder::DupeFinder::new(directories);
let results = checker.run();
for key in results.keys() {
    let result = results.get(key);
    if let Some(details) = result {
        println!("{} files of size {} bytes found with hash {}", details.files.len(), details.size, details.hash);
        for file in details.files.iter() {
            println!("{}", file);
        }
    }
}
```
## Recursive subfolder search
```
let directories = vec![String::from("./resources")];
let mut checker = dupefinder::DupeFinder::new_recursive(directories);
let results = checker.run();

for key in results.keys() {
    let result = results.get(key);
    if let Some(details) = result {
        println!("{} files of size {} bytes found with hash {}", details.files.len(), details.size, details.hash);
        for file in details.files.iter() {
            println!("{}", file);
        }
    }
}
```
## Specific file search
```
let directories = vec![String::from("./resources")];
let mut checker = dupefinder::DupeFinder::new(directories);
let results = checker.run_for_file(String::from("./test.txt"));

if let Ok(results) = results {
    match results {
       Some(duplicate) => {
           println!("{} files found", duplicate.files.len());
        },
       None => {
           println!("no matching files found");
       },
    }
};
```