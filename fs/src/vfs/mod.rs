mod dir;
mod filesystem;
mod inode;
mod io;
mod path;

pub mod error;
pub mod meta;

use core::fmt::Display;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

pub use dir::VfsDirEntry;
pub use filesystem::FileSystem;
pub use inode::VfsInode;
pub use path::VfsPath;

use self::error::{VfsErrorKind, VfsResult};

#[derive(Debug)]
pub struct VFS {
    fs: Box<dyn FileSystem>,
}

impl Display for VFS {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.fs)
    }
}

impl VFS {
    pub fn new(fs: impl FileSystem) -> VFS {
        VFS { fs: Box::new(fs) }
    }

    fn parse_path(path: &str) -> VfsResult<VfsPath> {
        if !path.starts_with('/') {
            return Err(VfsErrorKind::InvalidPath(path.to_string()).into());
        }

        Ok(VfsPath::from(path))
    }

    pub fn read_dir<T: AsRef<str>>(&self, path: T) -> VfsResult<Vec<Box<dyn VfsDirEntry>>> {
        let vpath = Self::parse_path(path.as_ref())?;
        self.fs.read_dir(vpath)
    }
}
