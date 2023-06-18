use core::slice::Iter;

use alloc::{string::String, vec::Vec};

#[derive(Debug)]
pub struct VfsPath {
    from_root: bool,
    inner: Vec<String>,
}

impl VfsPath {
    pub fn to_string(&self) -> String {
        let joined = self.inner.join("/");
        if self.from_root {
            alloc::format!("/{}", joined)
        } else {
            joined
        }
    }

    // pub fn iter(&self) -> Iter<String> {
    //     self.inner.iter()
    // }
}

impl<'a> Iterator for &'a VfsPath {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.iter().next().map(|x| x.as_str())
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
