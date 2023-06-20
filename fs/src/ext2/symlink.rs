use alloc::string::{String, ToString};

use crate::vfs::{
    error::{IOError, IOErrorKind, VfsResult},
    VfsPath,
};

use super::inode::Inode;

impl Inode {
    pub fn read_symlink(&self) -> String {
        self.read_disk_inode(|ext2_inode| {
            let symlink_len = ext2_inode.size();
            assert!(symlink_len <= 60, "Too large symlink: {}", symlink_len);
            let slice = unsafe {
                let start_ptr = (ext2_inode as *const _ as *const u8).add(40);
                core::slice::from_raw_parts(start_ptr, symlink_len)
            };
            String::from_utf8(slice.to_vec()).unwrap()
        })
    }

    pub fn symlink_target(&self, path: &VfsPath) -> VfsResult<VfsPath> {
        if !self.is_symlink() {
            return Err(IOError::new(IOErrorKind::NotASymlink)
                .with_path(path.to_string())
                .into());
        }
        Ok(VfsPath::from(self.read_symlink().as_str()))
    }
}
