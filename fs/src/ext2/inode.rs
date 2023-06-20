use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::vfs::error::{IOError, IOErrorKind, VfsResult};
use crate::vfs::meta::{VfsFileType, VfsMetadata, VfsTimeStamp};
use crate::vfs::{VfsDirEntry, VfsInode, VfsPath};
use crate::{block_device, vfs::meta::VfsPermissions};

use super::dir::Dir;
use super::layout::Ext2Layout;
use super::metadata::Ext2Metadata;
use super::{disk_inode::Ext2Inode, Address};

#[derive(Debug, Clone)]
pub struct Inode {
    address: Address,
    inode_id: usize,
    filetype: VfsFileType,

    parent_id: Option<usize>,
    layout: Option<Arc<Ext2Layout>>,
}
impl Inode {
    pub(crate) fn new(inode_id: usize, address: Address) -> Inode {
        let filetype = block_device::read(
            address.block_id(),
            address.offset(),
            |disk_inode: &Ext2Inode| disk_inode.filetype(),
        );

        Self {
            address,
            inode_id,
            filetype,

            parent_id: None,
            layout: None,
        }
    }

    pub(crate) fn with_parent(self, parent_id: usize) -> Self {
        Self {
            parent_id: Some(parent_id),
            ..self
        }
    }

    pub(crate) fn with_layout(self, layout: Arc<Ext2Layout>) -> Self {
        Self {
            layout: Some(layout),
            ..self
        }
    }

    pub fn inode_id(&self) -> usize {
        self.inode_id
    }

    pub fn parent_id(&self) -> usize {
        self.parent_id.unwrap()
    }

    pub fn parent_inode(&self) -> Inode {
        self.layout
            .as_ref()
            .unwrap()
            .inode_nth(self.parent_id(), self.layout())
    }

    pub fn layout(&self) -> Arc<Ext2Layout> {
        self.layout.clone().unwrap()
    }

    pub fn size(&self) -> usize {
        block_device::read(
            self.address.block_id(),
            self.address.offset(),
            |disk_inode: &Ext2Inode| disk_inode.size(),
        )
    }

    pub fn timestamp(&self) -> VfsTimeStamp {
        block_device::read(
            self.address.block_id(),
            self.address.offset(),
            |disk_inode: &Ext2Inode| disk_inode.timestamp(),
        )
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

    pub(crate) fn read_disk_inode<V>(&self, f: impl FnOnce(&Ext2Inode) -> V) -> V {
        block_device::read(self.block_id(), self.offset(), f)
    }

    pub(crate) fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut Ext2Inode) -> V) -> V {
        block_device::modify(self.block_id(), self.offset(), f)
    }

    pub(crate) fn sync_disk_inode(&self) {
        block_device::sync(self.block_id());
    }

    pub fn metadata(&self) -> Ext2Metadata {
        self.read_disk_inode(|ext2_inode| {
            Ext2Metadata::new(
                ext2_inode.filetype(),
                ext2_inode.permissions(),
                ext2_inode.size(),
                ext2_inode.timestamp(),
                ext2_inode.uid(),
                ext2_inode.gid(),
                ext2_inode.hard_links(),
            )
        })
    }
}

impl VfsInode for Inode {
    fn metadata(&self) -> Box<dyn VfsMetadata> {
        // 有趣的是, 如果函数重名(比如这里的 metadata 和 Inode 的 metadata)
        // 并不会发生冲突, 而是结构体方法优先
        Box::new(self.metadata())
    }

    fn read_symlink(&self) -> String {
        self.read_symlink()
    }
}
