use core::{fmt::Display, slice::Iter};

use alloc::{string::{String, ToString}, vec::Vec};
use core::ops::Deref;

#[derive(Debug)]
pub struct VfsPath {
    from_root: bool,
    inner: Vec<String>,
}

impl VfsPath {
    pub fn empty(from_root: bool) -> VfsPath {
        VfsPath {
            from_root,
            inner: Vec::new(),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.inner.iter().map(|x| x.as_str())
    }

    pub fn is_from_root(&self) -> bool {
        self.from_root
    }

    pub fn push(&mut self, next: &str) {
        self.inner.push(next.to_string());
    }
}

impl Display for VfsPath {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let joined = self.inner.join("/");
        if self.from_root {
            write!(f, "/{}", joined)
        } else {
            write!(f, "{}", joined)
        }
    }
}

impl Deref for VfsPath {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<&str> for VfsPath {
    fn from(path: &str) -> Self {
        let from_root = path.starts_with('/');

        Self {
            from_root,
            inner: path
                .split('/')
                .filter(|mid| !mid.is_empty())
                .map(String::from)
                .collect(),
        }
    }
}
