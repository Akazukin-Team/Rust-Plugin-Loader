use crate::matcher::Matcher;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io::{Error as IoError, ErrorKind, Result};
use std::path::{Path, PathBuf};

pub fn find_files(dir: &Path, matcher: &dyn Matcher<Item = PathBuf>) -> Result<HashSet<PathBuf>> {
    if dir.is_dir() {
        let mut results = HashSet::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                results.extend(find_files(&path, matcher)?);
            } else if path.is_file() {
                if matcher.matches(&path) {
                    results.insert(path);
                }
            }
        }
        Ok(results)
    } else {
        return Err(IoError::new(
            ErrorKind::NotADirectory,
            dir.to_str().unwrap(),
        ));
    }
}
