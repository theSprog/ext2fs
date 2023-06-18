use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::vfs::error::{IOError, IOErrorKind, VfsResult};
use crate::vfs::meta::VfsFileType;
use crate::vfs::{VfsDirEntry, VfsInode, VfsPath};
use crate::{block_device, vfs::meta::VfsPermissions};

use super::dir::Dir;
use super::layout::Ext2Layout;
use super::{disk_inode::Ext2Inode, Address};

#[derive(Debug, Clone)]
pub struct Inode {
    address: Address,
    self_id: usize,
    parent_id: Option<usize>,
    layout: Arc<Ext2Layout>,
    filetype: VfsFileType,
    permissions: VfsPermissions,
}
impl Inode {
    pub(crate) fn new(self_id: usize, address: Address, layout: Arc<Ext2Layout>) -> Inode {
        let (filetype, permissions) = block_device::read(
            address.block_id(),
            address.offset(),
            |disk_inode: &Ext2Inode| (disk_inode.filetype(), disk_inode.permissions()),
        );

        Self {
            address,
            self_id,
            parent_id: None,
            layout,
            filetype,
            permissions,
        }
    }

    pub(crate) fn with_parent(self, parent_id: usize) -> Self {
        Self {
            parent_id: Some(parent_id),
            ..self
        }
    }

    pub fn inode_id(&self) -> usize {
        self.self_id
    }

    pub fn parent_id(&self) -> usize {
        self.parent_id.unwrap()
    }

    pub fn is_file(&self) -> bool {
        self.filetype.is_file()
    }
    pub fn is_dir(&self) -> bool {
        self.filetype.is_dir()
    }
    pub fn is_symlink(&self) -> bool {
        self.filetype.is_symlink()
    }

    fn block_id(&self) -> usize {
        self.address.block_id()
    }
    fn offset(&self) -> usize {
        self.address.offset()
    }

    fn read_disk_inode<V>(&self, f: impl FnOnce(&Ext2Inode) -> V) -> V {
        block_device::read(self.block_id(), self.offset(), f)
    }

    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut Ext2Inode) -> V) -> V {
        block_device::modify(self.block_id(), self.offset(), f)
    }

    fn sync_disk_inode<V>(&self) {
        block_device::sync(self.block_id());
    }

    pub(crate) fn walk(&self, path: &VfsPath) -> VfsResult<Inode> {
        let last = self.goto_last(path)?;
        if last.is_symlink() {
            self.goto_last(&last.symlink_target()?)
        } else {
            Ok(last)
        }
    }

    fn goto_last(&self, path: &VfsPath) -> VfsResult<Inode> {
        let mut path = path.clone();
        let next = path.next();
        let current_inode = self.clone();

        loop {
            match next {
                None => {
                    return Ok(current_inode.clone());
                }
                Some(name) => {
                    todo!()
                }
            }
        }
    }

    fn symlink_target(&self) -> VfsResult<VfsPath> {
        if !self.is_symlink() {
            return Err(IOError::new(IOErrorKind::NotASymlink).into());
        }

        todo!()
    }

    pub(crate) fn read_dir(&self) -> VfsResult<Vec<Box<dyn VfsDirEntry>>> {
        if !self.is_dir() {
            return Err(IOError::new(IOErrorKind::NotADirectory).into());
        }

        self.read_disk_inode(|ext2_inode| {
            let dir = Dir::from_inode(self.inode_id(), ext2_inode, self.layout.clone());
            Ok(dir
                .entries()
                .into_iter()
                .map(|x| x as Box<dyn VfsDirEntry>)
                .collect())
        })
    }
}

impl VfsInode for Inode {}
