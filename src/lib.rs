pub mod detector;
pub mod error;

#[cfg(test)]
mod tests;

use crate::error::{Error, ErrorKind};
use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

/// Struct [`JavaRuntime`] Represents a java runtime in specific path.
///
/// To detect java runtimes from specific path in filesystem, see [`crate::detector`]
///
/// ## Examples
///
/// ```rs
/// JavaRuntime::from_java_exe(r"D:\java\jdk-17.0.4.1\bin\java.exe".as_ref());
/// JavaRuntime::from_java_exe(r"../../runtimes/jdk-1.8.0_291/bin/java".as_ref());
/// ```
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
    pub fn from_java_exe(path: &Path) -> Result<Self, Error> {
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
    /// ## Argument
    ///
    /// * `os` Got from [`env::consts::OS`]
    /// * `path` The path of java executable file, can be either relative or absolute
    /// * `version_string` can be like `"17.0.4.1"` or the output of command `java -version`
    ///
    ///
    /// ## Command `java -version` output Examples
    ///
    /// ```txt
    /// java version "1.8.0_333"
    /// Java(TM) SE Runtime Environment (build 1.8.0_333-b02)
    /// Java HotSpot(TM) 64-Bit Server VM (build 25.333-b02, mixed mode)
    /// ```
    ///
    /// ```txt
    /// java version "17.0.4.1" 2022-08-18 LTS
    /// Java(TM) SE Runtime Environment (build 17.0.4.1+1-LTS-2)
    /// Java HotSpot(TM) 64-Bit Server VM (build 17.0.4.1+1-LTS-2, mixed mode, sharing)
    /// ```
    ///
    /// ```txt
    /// java version "21.0.3" 2024-04-16 LTS
    /// Java(TM) SE Runtime Environment (build 21.0.3+7-LTS-152)
    /// Java HotSpot(TM) 64-Bit Server VM (build 21.0.3+7-LTS-152, mixed mode, sharing)
    /// ```
    pub fn new(os: &str, path: &Path, version_string: &str) -> Result<Self, Error> {
        let version_string = Self::extract_version(version_string)?;
        Version::from_str(&version_string)
            .map_err(|e| Error::new(ErrorKind::SemverParseFailed(e)))?;
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
    /// ## Examples
    ///
    /// * `D:\Java\jdk-17.0.4.1\bin\java.exe` (Windows, absolute)
    /// * `../../runtimes/jdk-1.8.0_291/bin/java` (Linux, relative)
    pub fn get_executable(&self) -> &Path {
        &self.path
    }
    /// Is the path relative
    pub fn is_relative(&self) -> bool {
        self.path.is_relative()
    }
    /// Is the path absolute
    pub fn is_absolute(&self) -> bool {
        self.path.is_absolute()
    }
    pub fn get_version(&self) -> Version {
        Version::from_str(&self.version_string).unwrap()
    }
    /// Get the version string
    ///
    /// ## Examples
    ///
    /// * `"1.8.0_333"`
    /// * `"17.0.4.1"`
    pub fn get_version_string(&self) -> &str {
        &self.version_string
    }

    /// Check if this is the same os as current
    pub fn is_same_os(&self) -> bool {
        self.os == env::consts::OS
    }

    /// Create a new [`JavaRuntime`] with absolute path.
    ///
    /// ## Error
    ///
    /// * Cannot find home directory
    pub fn to_absolute(&self) -> Result<Self, Error> {
        let cwd = env::current_dir().or(Err(Error::new(ErrorKind::HomeDirNotFound)))?;
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
            Err(Error::new(ErrorKind::GettingJavaVersionFailed(self.path.clone())))
        }
    }

    /// Test if this runtime is available currently
    ///
    /// It executes command `java -version` to see if it works
    pub fn is_available(&self) -> bool {
        self.is_same_os() && Self::from_java_exe(&self.path).is_ok()
    }

    /// Parse version string
    ///
    /// ## Return
    ///
    /// `(version_string, version_major)`
    ///
    /// ## Examples
    ///
    /// ```rs
    /// extract_version("1.8.0_333")     // Ok("1.8.0_333", 1)
    /// extract_version("17.0.4.1")     // Ok("17.0.4.1", 17)
    /// extract_version("\"17.0.4.1\"") // Ok("17.0.4.1", 17)
    /// extract_version("java version \"17.0.4.1\"") // Ok("17.0.4.1", 17)
    /// extract_version("17")           // Err("Bad java version string: '\"17\"'")
    /// ```
    fn extract_version(version_string: &str) -> Result<String, Error> {
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

    /// ## Examples
    /// * `java.exe` (windows)
    /// * `java` (linux)
    fn get_java_executable_name() -> OsString {
        let mut java_exe = OsString::from("java");
        java_exe.push(env::consts::EXE_SUFFIX);
        java_exe
    }
}

impl PartialEq for JavaRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.os == other.os && self.path == other.path
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Clone for JavaRuntime {
    fn clone(&self) -> Self {
        Self {
            os: self.os.clone(),
            path: self.path.clone(),
            version_string: self.version_string.clone(),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.os = source.os.clone();
        self.path = source.path.clone();
        self.version_string = source.version_string.clone();
    }
}
