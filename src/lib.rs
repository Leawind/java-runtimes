//! `java-runtimes` is a rust library for detecting java runtimes in current system.
//!
//! * To detect java runtimes, see [`detector`]
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

pub mod detector;
pub mod error;

use crate::error::{Error, ErrorKind};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Struct [`JavaRuntime`] Represents a java runtime in specific path.
///
/// To detect java runtimes from specific path, see [`detector`]
#[derive(Serialize, Deserialize, Debug)]
pub struct JavaRuntime {
    os: String,
    path: PathBuf,
    version_string: String,
}

impl JavaRuntime {
    /// Used to match the version string in the command output
    ///
    const VERSION_PATTERN: &'static str = r#".*"((\d+)\.(\d+)([\d._]+)?)".*"#;
    /// Create a [`JavaRuntime`] object from the path of java executable file
    ///
    /// It executes command `java -version` to get the version information
    ///
    /// # Parameters
    ///
    /// * `path` Path to java executable file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use java_runtimes::JavaRuntime;
    ///
    /// let _ = JavaRuntime::from_executable(r"D:\java\jdk-17.0.4.1\bin\java.exe".as_ref());
    /// let _ = JavaRuntime::from_executable(r"../../runtimes/jdk-1.8.0_291/bin/java".as_ref());
    /// ```
    pub fn from_executable(path: &Path) -> Result<Self, Error> {
        let mut java = Self {
            os: env::consts::OS.to_string(),
            path: path.to_path_buf(),
            version_string: String::new(),
        };
        java.update()?;
        Ok(java)
    }

    /// Mannually create a [`JavaRuntime`] instance, without checking if it's available
    ///
    /// # Parameters
    ///
    /// * `os` Got from [`env::consts::OS`]
    /// * `path` The path of java executable file, can be either relative or absolute
    /// * `version_string` can be like `"17.0.4.1"` or the output of command `java -version`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use java_runtimes::JavaRuntime;
    /// use std::env;
    /// use std::path::Path;
    ///
    /// let java_exe_path = Path::new("../java/jdk-17.0.4.1/bin/java");
    /// let version_outputs = r#"java version "17.0.4.1" 2022-08-18 LTS
    /// Java(TM) SE Runtime Environment (build 17.0.4.1+1-LTS-2)
    /// Java HotSpot(TM) 64-Bit Server VM (build 17.0.4.1+1-LTS-2, mixed mode, sharing)
    /// "#;
    /// let runtime = JavaRuntime::new(env::consts::OS, java_exe_path, version_outputs).unwrap();
    /// assert_eq!(runtime.get_version_string(), "17.0.4.1");
    /// assert!(runtime.is_same_os());
    /// ```
    pub fn new(os: &str, path: &Path, version_string: &str) -> Result<Self, Error> {
        let version_string = Self::extract_version(version_string)?;
        Ok(Self {
            os: os.to_string(),
            path: path.to_path_buf(),
            version_string: version_string.to_string(),
        })
    }

    /// Get the operating system of the java runtime
    ///
    /// The os string comes from [`env::consts::OS`] when this object was created.
    pub fn get_os(&self) -> &str {
        &self.os
    }
    pub fn is_windows(&self) -> bool {
        self.os == "windows"
    }
    /// Get the path of java executable file
    ///
    /// It can be absolute or relative, depends on how you created it.
    ///
    /// # Examples
    ///
    /// * `D:\Java\jdk-17.0.4.1\bin\java.exe` (Windows, absolute)
    /// * `../../runtimes/jdk-1.8.0_291/bin/java` (Linux, relative)
    pub fn get_executable(&self) -> &Path {
        &self.path
    }

    /// Returns `true` if the `Path` has a root.
    ///
    /// Refer to [`Path::has_root`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// use java_runtimes::JavaRuntime;
    ///
    /// let runtime = JavaRuntime::new("linux", "/jdk/bin/java".as_ref(), "21.0.3").unwrap();
    /// assert!(runtime.has_root());
    ///
    /// let runtime = JavaRuntime::new("windows", r"D:\jdk\bin\java.exe".as_ref(), "21.0.3").unwrap();
    /// assert!(runtime.has_root());
    ///
    /// let runtime = JavaRuntime::new("linux", "../jdk/bin/java".as_ref(), "21.0.3").unwrap();
    /// assert!(!runtime.has_root());
    ///
    /// let runtime = JavaRuntime::new("windows", r"..\jdk\bin\java.exe".as_ref(), "21.0.3").unwrap();
    /// assert!(!runtime.has_root());
    /// ```
    pub fn has_root(&self) -> bool {
        self.path.has_root()
    }

    /// Get the version string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use java_runtimes::JavaRuntime;
    ///
    /// let runtime = JavaRuntime::new("linux", "/jdk/bin/java".as_ref(), "21.0.3").unwrap();
    /// assert_eq!(runtime.get_version_string(), "21.0.3");
    /// ```
    pub fn get_version_string(&self) -> &str {
        &self.version_string
    }

    /// Check if this is the same os as current
    pub fn is_same_os(&self) -> bool {
        self.os == env::consts::OS
    }

    /// Create a new [`JavaRuntime`] with absolute path.
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if the current working directory value is invalid. Refer to [`env::current_dir`]
    ///
    /// Possible cases:
    ///
    /// * Current directory does not exist.
    /// * There are insufficient permissions to access the current directory.
    pub fn to_absolute(&self) -> Result<Self, Error> {
        let cwd = env::current_dir().or(Err(Error::new(ErrorKind::InvalidWorkDir)))?;
        let path_absolute = self.path.join(cwd);
        let new_runtime = Self::new(&self.os, &path_absolute, &self.version_string)?;
        Ok(new_runtime)
    }

    /// Try executing `java -version` and parse the output to get the version.
    ///
    /// If success, it will update the version value in this [`JavaRuntime`] instance.
    pub fn update(&mut self) -> Result<(), Error> {
        if !Self::looks_like_java_executable_file(&self.path) {
            return Err(Error::new(ErrorKind::LooksNotLikeJavaExecutableFile(
                self.path.clone(),
            )));
        }

        let output = Command::new(&self.path)
            .arg("-version")
            .output()
            .map_err(|err| Error::new(ErrorKind::JavaOutputFailed(err)))?;

        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stderr).to_string();
            self.version_string = Self::extract_version(&version_output)?;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::GettingJavaVersionFailed(
                self.path.clone(),
            )))
        }
    }

    /// Test if this runtime is available currently
    ///
    /// It executes command `java -version` to see if it works
    pub fn is_available(&self) -> bool {
        self.is_same_os() && Self::from_executable(&self.path).is_ok()
    }

    /// Parse version string
    ///
    /// # Return
    ///
    /// `(version_string, version_major)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use java_runtimes::JavaRuntime;
    ///
    /// assert_eq!(JavaRuntime::extract_version("1.8.0_333").unwrap(), "1.8.0_333");
    /// assert_eq!(JavaRuntime::extract_version("17.0.4.1").unwrap(), "17.0.4.1");
    /// assert_eq!(JavaRuntime::extract_version("\"17.0.4.1").unwrap(), "17.0.4.1");
    /// assert_eq!(JavaRuntime::extract_version("java version \"17.0.4.1\"").unwrap(), "17.0.4.1");
    /// ```
    pub fn extract_version(version_string: &str) -> Result<String, Error> {
        Ok(Regex::new(Self::VERSION_PATTERN)
            .unwrap()
            .captures(&format!("\"{}\"", &version_string))
            .ok_or(Error::new(ErrorKind::NoJavaVersionStringFound))?
            .get(1)
            .ok_or(Error::new(ErrorKind::NoJavaVersionStringFound))?
            .as_str()
            .to_string())
    }

    /// Check if the given path looks like a java executable file
    ///
    /// The file must exists.
    ///
    /// The given path must be `**/bin/java.exe` in windows, or `**/bin/java` in unix
    fn looks_like_java_executable_file(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        // to absolute
        let path_absolute = match path.canonicalize() {
            Ok(path) => path,
            _ => return false,
        };
        // check file name
        if let Some(file_name) = path_absolute.file_name() {
            if file_name == Self::get_java_executable_name() {
                // check parent name
                if let Some(parent) = path_absolute.parent() {
                    if let Some(dir_name) = parent.file_name() {
                        if dir_name == "bin" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// # Examples
    /// * `java.exe` (windows)
    /// * `java` (linux)
    fn get_java_executable_name() -> OsString {
        let mut java_exe = OsString::from("java");
        java_exe.push(env::consts::EXE_SUFFIX);
        java_exe
    }
}
impl Clone for JavaRuntime {
    /// # Examples
    ///
    /// ```rust
    /// use java_runtimes::JavaRuntime;
    ///
    /// let r1 = JavaRuntime::new("linux", "/jdk/bin/java".as_ref(), "21.0.3").unwrap();
    /// let r2 = r1.clone();
    ///
    /// assert_eq!(r1, r2);
    /// ```
    fn clone(&self) -> Self {
        Self {
            os: self.os.clone(),
            path: self.path.clone(),
            version_string: self.version_string.clone(),
        }
    }
    /// # Examples
    ///
    /// ```rust
    /// use java_runtimes::JavaRuntime;
    ///
    /// let mut r1 = JavaRuntime::new("windows", "/jdk/bin/java".as_ref(), "21.0.3").unwrap();
    /// let r2 = JavaRuntime::new("windows", r"D:\jdk\bin\java.exe".as_ref(), "21.0.3").unwrap();
    ///
    /// r1.clone_from(&r2);
    /// assert_eq!(r1, r2);
    /// ```
    fn clone_from(&mut self, source: &Self) {
        self.os = source.os.clone();
        self.path = source.path.clone();
        self.version_string = source.version_string.clone();
    }
}

impl PartialEq for JavaRuntime {
    /// # Examples
    ///
    /// ```rust
    /// use java_runtimes::JavaRuntime;
    ///
    /// let r1 = JavaRuntime::new("linux", "/jdk/bin/java".as_ref(), "21.0.3").unwrap();
    /// let r2 = JavaRuntime::new("linux", "/jdk/bin/java".as_ref(), "21.0.3").unwrap();
    /// let r3 = JavaRuntime::new("windows", r"D:\jdk\bin\java.exe".as_ref(), "21.0.3").unwrap();
    /// let r4 = JavaRuntime::new("windows", r"D:\jdk-17\bin\java.exe".as_ref(), "21.0.3").unwrap();
    ///
    /// assert_eq!(r1, r2);
    /// assert_ne!(r1, r3);
    /// assert_ne!(r2, r3);
    /// assert_ne!(r2, r4);
    /// assert_ne!(r3, r4);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.os == other.os && self.path == other.path
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
