//! This module contains the logic for filtering files based on include and exclude patterns.

use colored::*;
use glob::Pattern;
use log::{debug, error};
use std::fs;
use std::path::Path;
use regex::Regex;

/// Determines whether a file should be included based on include and exclude patterns.
///
/// # Arguments
///
/// * `path` - The path to the file to be checked.
/// * `include_patterns` - A slice of strings representing the include patterns.
/// * `exclude_patterns` - A slice of strings representing the exclude patterns.
/// * `include_priority` - A boolean indicating whether to give priority to include patterns if both include and exclude patterns match.
///
/// # Returns
///
/// * `bool` - `true` if the file should be included, `false` otherwise.
pub fn should_include_file(
    path: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
    include_priority: bool,
) -> bool {
    // ~~~ Clean path ~~~
    let canonical_path = match fs::canonicalize(path) {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to canonicalize path: {}", e);
            return false;
        }
    };
    let path_str = canonical_path.to_str().unwrap_or("");
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // ~~~ Check patterns ~~~
    let included = include_patterns.iter().any(|pattern| {
        // Try as glob pattern first
        if let Ok(glob) = Pattern::new(pattern) {
            if glob.matches(path_str) {
                return true;
            }
        }

        // Try as simple wildcard pattern
        if let Some(wildcard_pattern) = pattern.strip_suffix('*') {
            if file_name.starts_with(wildcard_pattern) {
                return true;
            }
        }

        // Try exact match
        file_name == pattern
    });

    let excluded = exclude_patterns
        .iter()
        .any(|pattern| Pattern::new(pattern).unwrap().matches(path_str));

    // ~~~ Decision ~~~
    let result = match (included, excluded) {
        (true, true) => include_priority, // If both include and exclude patterns match, use the include_priority flag
        (true, false) => true,            // If the path is included and not excluded, include it
        (false, true) => false,           // If the path is excluded, exclude it
        (false, false) => include_patterns.is_empty(), // If no include patterns are provided, include everything
    };

    debug!(
        "Checking path: {:?}, {}: {}, {}: {}, decision: {}",
        path_str,
        "included".bold().green(),
        included,
        "excluded".bold().red(),
        excluded,
        result
    );
    result
}
