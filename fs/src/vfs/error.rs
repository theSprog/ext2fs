//! Error and Result definitions

use super::io;
use crate::alloc::string::ToString;
use core::{error, fmt};

use alloc::string::String;

/// The only way to create a VfsError is via a VfsErrorKind
/// The result type of this crate
pub type VfsResult<T> = core::result::Result<T, VfsError>;

/// The error type of this crate
#[derive(Debug)]
pub struct VfsError {
    path: String,
    kind: VfsErrorKind,
    context: String,
}

/// This conversion implements certain normalizations
impl From<VfsErrorKind> for VfsError {
    fn from(kind: VfsErrorKind) -> Self {
        Self {
            path: "PATH NOT FILLED BY VFS LAYER".into(),
            kind,
            context: "An error occured".into(),
        }
    }
}

impl From<IOError> for VfsError {
    fn from(err: IOError) -> Self {
        Self::from(VfsErrorKind::IoError(err))
    }
}

impl VfsError {
    // Path filled by the VFS crate rather than the implementations
    pub(crate) fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    pub fn with_context<C, F>(mut self, context: F) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.context = context().to_string();
        self
    }

    pub fn kind(&self) -> &VfsErrorKind {
        &self.kind
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}

impl fmt::Display for VfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} for '{}': {}", self.context, self.path, self.kind())
    }
}

/// The kinds of errors that can occur
#[derive(Debug)]
pub enum VfsErrorKind {
    /// A generic I/O error
    ///
    /// Certain standard I/O errors are normalized to their VfsErrorKind counterparts
    IoError(IOError),

    // FSError(FSError),
    /// The file or directory at the given path could not be found
    FileNotFound,

    /// The given path is invalid, e.g. because contains '.' or '..'
    InvalidPath(String),

    /// There is already a directory at the given path
    DirectoryExists,

    /// There is already a file at the given path
    FileExists,

    /// Functionality not supported by this filesystem
    NotSupported,

    /// Generic error variant
    Other(String),
}

impl fmt::Display for VfsErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VfsErrorKind::IoError(err) => {
                write!(f, "IO error: {:?}", err)
            }
            // VfsErrorKind::FSError(err) => {
            //     write!(f, "FS error: {:?}", err)
            // }
            VfsErrorKind::FileNotFound => {
                write!(f, "The file or directory could not be found")
            }
            VfsErrorKind::InvalidPath(path) => {
                write!(f, "The path is invalid: {}", path)
            }
            VfsErrorKind::Other(msg) => {
                write!(f, "FileSystem error: {}", msg)
            }
            VfsErrorKind::NotSupported => {
                write!(f, "Functionality not supported by this filesystem")
            }
            VfsErrorKind::DirectoryExists => {
                write!(f, "Directory already exists")
            }
            VfsErrorKind::FileExists => {
                write!(f, "File already exists")
            }
        }
    }
}

#[derive(Debug)]
pub struct IOError {
    kind: IOErrorKind,
}

impl IOError {
    pub fn new(kind: IOErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
pub enum IOErrorKind {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    NotADirectory,
    NotAFile,
    NotASymlink,
    DirectoryNotEmpty,
    IsADirectory,
    TooLargeFile,
    TooLongFileName,
    TooManyLinks,
    InvalidFilename,
}
