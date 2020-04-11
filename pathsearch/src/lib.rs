//! This crate provides functions that can be used to search for an
//! executable based on the PATH environment on both POSIX and Windows
//! systems.
//!
//! `find_executable_in_path` is the most convenient function exported
//! by this crate; given the name of an executable, it will yield the
//! absolute path of the first matching file.
//!
//! ```
//! use pathsearch::find_executable_in_path;
//!
//! if let Some(exe) = find_executable_in_path("ls") {
//!   println!("Found ls at {}", exe.display());
//! }
//! ```
//!
//! `PathSearcher` is platform-independent struct that encompasses the
//! path searching algorithm used by `find_executable_in_path`.  Construct
//! it by passing in the PATH and PATHEXT (for Windows) environment variables
//! and iterate it to incrementally produce all candidate results.  This
//! is useful when implementing utilities such as `which` that want to show
//! all possible paths.
//!
//! ```
//! use pathsearch::PathSearcher;
//! use std::ffi::OsString;
//!
//! let path = std::env::var_os("PATH");
//! let path_ext = std::env::var_os("PATHEXT");
//!
//! for exe in PathSearcher::new(
//!     "zsh",
//!     path.as_ref().map(OsString::as_os_str),
//!     path_ext.as_ref().map(OsString::as_os_str),
//! ) {
//!     println!("{}", exe.display());
//! }
//! ```
//!
//! `SimplePathSearcher` is a simple iterator that can be used to search
//! an arbitrary path for an arbitrary file that doesn't have to be executable.
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

/// SimplePathSearcher is an iterator that yields candidate PathBuf inthstances
/// generated from searching the supplied path string following the
/// standard rules: explode path by the system path separator character
/// and then for each entry, concatenate the candidate command and test
/// whether that is a file.
pub struct SimplePathSearcher<'a> {
    path_iter: std::env::SplitPaths<'a>,
    command: &'a OsStr,
}

impl<'a> SimplePathSearcher<'a> {
    /// Create a new SimplePathSearcher that will yield candidate paths for
    /// the specified command
    pub fn new<T: AsRef<OsStr> + ?Sized>(command: &'a T, path: Option<&'a OsStr>) -> Self {
        let path = path.unwrap_or_else(|| OsStr::new(""));
        let path_iter = std::env::split_paths(path);
        let command = command.as_ref();
        Self { path_iter, command }
    }
}

impl<'a> Iterator for SimplePathSearcher<'a> {
    type Item = PathBuf;

    /// Returns the next candidate path
    fn next(&mut self) -> Option<PathBuf> {
        loop {
            let entry = self.path_iter.next()?;
            let candidate = entry.join(self.command);

            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
}

#[cfg(unix)]
pub type PathSearcher<'a> = unix::ExecutablePathSearcher<'a>;

#[cfg(windows)]
pub type PathSearcher<'a> = windows::WindowsPathSearcher<'a>;

/// Resolves the first matching candidate command from the current
/// process environment using the platform appropriate rules.
/// On Unix systems this will search the PATH environment variable
/// for an executable file.
/// On Windows systems this will search each entry in PATH and
/// return the first file that has an extension listed in the PATHEXT
/// environment variable.
pub fn find_executable_in_path<O: AsRef<OsStr> + ?Sized>(command: &O) -> Option<PathBuf> {
    let path = std::env::var_os("PATH");
    let path_ext = std::env::var_os("PATHEXT");
    PathSearcher::new(
        command,
        path.as_ref().map(OsString::as_os_str),
        path_ext.as_ref().map(OsString::as_os_str),
    )
    .next()
}
