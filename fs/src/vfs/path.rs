use alloc::{string::String, vec::Vec};

#[derive(Debug)]
pub struct VfsPath {
    inner: Vec<String>,
}

impl From<&str> for VfsPath {
    fn from(path: &str) -> Self {
        Self {
            inner: path
                .split('/')
                .filter(|mid| !mid.is_empty())
                .map(String::from)
                .collect(),
        }
    }
}
