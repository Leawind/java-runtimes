//! This module provides functions for detecting available Java runtimes from given path(s).
//!
//! The detected java runtimes are represented by the [`JavaRuntime`] struct.
//!
//! # Examples
//!
//! Detect Java runtime from environment variables
//!
//! ```rust
//! use java_runtimes::detector;
//!
//! let runtimes = detector::detect_java_in_environments();
//! println!("Detected Java runtimes: {:?}", runtimes);
//! ```
//!
//! Detect Java runtimes recursively within a path
//!
//! ```rust
//! use java_runtimes::detector;
//!
//! let runtimes = detector::detect_java("/usr".as_ref(), 2);
//! println!("Detected Java runtimes: {:?}", runtimes);
//! ```
//!
//! Detect Java runtimes recursively within multiple paths
//!
//! ```rust
//! use java_runtimes::detector;
//!
//! let runtimes = detector::detect_java_in_paths(&[
//!     "/usr".as_ref(),
//!     "/opt".as_ref(),
//! ], 2);
//! println!("Detected Java runtimes in multiple paths: {:?}", runtimes);
//! ```

use crate::JavaRuntime;
use std::path::Path;
use walkdir::WalkDir;

/// Detects available Java runtimes within the specified path up to a maximum depth.
///
/// # Parameters
///
/// * `max_depth`: Maximum depth to search for Java runtimes (see [`WalkDir::max_depth`]).
///
/// # Returns
///
/// A vector containing all detected Java runtimes.
pub fn detect_java(path: &Path, max_depth: usize) -> Vec<JavaRuntime> {
    let mut runtimes: Vec<JavaRuntime> = vec![];
    gather_java(&mut runtimes, path, max_depth);
    runtimes
}

/// Detects available Java runtimes within the specified path and appends them to the given vector.
///
/// # Parameters
///
/// * `runtimes`: Vector to contain detected Java runtimes.
/// * `path`: The path to search for Java runtimes.
/// * `max_depth`: Maximum depth to search for Java runtimes (see [`WalkDir::max_depth`]).
///
/// # Returns
///
/// The number of new Java runtimes added to the vector.
pub fn gather_java(runtimes: &mut Vec<JavaRuntime>, path: &Path, max_depth: usize) -> usize {
    if path.is_file() {
        if let Some(runtime) = detect_java_bin_dir(path) {
            runtimes.push(runtime);
            return 1;
        }
    }

    let entries = WalkDir::new(path)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok);

    let begin_count = runtimes.len();

    for entry in entries {
        let path = entry.path();
        if let Some(runtime) = detect_java_bin_dir(path) {
            runtimes.push(runtime);
        }
    }
    runtimes.len() - begin_count
}

/// Detects available Java runtimes from environment variables.
///
/// It searches java runtime in paths below:
///
/// * `JAVA_HOME`
/// * `JAVA_ROOT`
/// * `JDK_HOME`
/// * `JRE_HOME`
/// * `PATH`
pub fn detect_java_in_environments() -> Vec<JavaRuntime> {
    let mut runtimes: Vec<JavaRuntime> = vec![];

    let mut gather_env = |var_name: &str| {
        if let Ok(env_java_home) = std::env::var(var_name) {
            gather_java(&mut runtimes, env_java_home.as_ref(), 1);
        }
    };

    gather_env("JAVA_HOME");
    gather_env("JAVA_ROOT");
    gather_env("JDK_HOME");
    gather_env("JRE_HOME");

    if let Ok(env_path) = std::env::var("PATH") {
        let paths = env_path
            .split(r":|;")
            .map(Path::new)
            .collect::<Vec<&Path>>();
        gather_java_in_paths(&mut runtimes, &paths, 1);
    }
    runtimes
}

/// Detects available Java runtimes within multiple paths up to a maximum depth.
///
/// # Parameters
///
/// * `paths`: The paths to search for Java runtimes.
/// * `max_depth`: Maximum depth to search for Java runtimes (see [`WalkDir::max_depth`]).
///
/// # Returns
///
/// A vector containing all detected Java runtimes.
pub fn detect_java_in_paths<'a>(paths: &[&Path], max_depth: usize) -> Vec<JavaRuntime> {
    let mut runtimes: Vec<JavaRuntime> = vec![];
    for &path in paths {
        gather_java(&mut runtimes, path, max_depth);
    }
    runtimes
}

/// Detects available Java runtimes within multiple paths up to a maximum depth and appends them to the given vector.
///
/// # Parameters
///
/// * `runtimes`: Vector to contain detected Java runtimes.
/// * `paths`: The paths to search for Java runtimes.
/// * `max_depth`: Maximum depth to search for Java runtimes (see [`WalkDir::max_depth`]).
///
/// # Returns
///
/// The number of new Java runtimes added to the vector.
pub fn gather_java_in_paths<'a>(
    runtimes: &mut Vec<JavaRuntime>,
    paths: &[&Path],
    max_depth: usize,
) -> usize {
    paths
        .iter()
        .map(|&path| gather_java(runtimes, path, max_depth))
        .sum::<usize>()
}

/// Attempts to detect a Java runtime from the given path.
///
/// # Returns
///
/// * `Some(JavaRuntime)` if the given path points to an available Java executable file.
/// * `None` if the given path is not an available Java executable file.
pub fn detect_java_exe(path: &Path) -> Option<JavaRuntime> {
    JavaRuntime::from_executable(path).map_or(None, |r| Some(r))
}

/// Attempts to detect a Java runtime from the given directory path.
///
/// # Returns
///
/// * `Some(JavaRuntime)` if the given path is a directory containing the Java executable file.
/// * `None` if the given path is not a directory containing the Java executable file.
pub fn detect_java_bin_dir(bin_dir: &Path) -> Option<JavaRuntime> {
    detect_java_exe(&bin_dir.join(JavaRuntime::get_java_executable_name()))
}

/// Attempts to detect a Java runtime from the given Java home directory path.
///
/// # Returns
///
/// * `Some(JavaRuntime)` if the given path is a directory containing the `bin` subdirectory with the Java executable file.
/// * `None` if the given path is not a directory containing the `bin` subdirectory with the Java executable file.
pub fn detect_java_home_dir(java_home: &Path) -> Option<JavaRuntime> {
    detect_java_bin_dir(&java_home.join("bin"))
}
