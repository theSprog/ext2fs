use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::__Deref;

use crate::vfs::error::{IOError, IOErrorKind, VfsResult};
use crate::vfs::meta::{VfsFileType, VfsMetadata, VfsTimeStamp};
use crate::vfs::{VfsDirEntry, VfsInode, VfsPath};
use crate::{block_device, vfs::meta::VfsPermissions};

use super::address::Address;
use super::allocator::Ext2Allocator;
use super::dir::Dir;
use super::disk_inode::Ext2Inode;
use super::layout::Ext2Layout;
use super::metadata::Ext2Metadata;

#[derive(Debug, Clone)]
pub struct Inode {
    address: Address,
    inode_id: usize,
    filetype: VfsFileType,

    layout: Arc<Ext2Layout>,
    allocator: Arc<Ext2Allocator>,

    parent_id: Option<usize>,
}
impl Inode {
    pub(crate) fn new(
        inode_id: usize,
        address: Address,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Ext2Allocator>,
    ) -> Inode {
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
            layout,
            allocator,
        }
    }

    pub(crate) fn with_parent(self, parent_id: usize) -> Self {
        Self {
            parent_id: Some(parent_id),
            ..self
        }
    }

    pub fn inode_id(&self) -> usize {
        self.inode_id
    }

    pub fn parent_id(&self) -> usize {
        self.parent_id.unwrap()
    }

    pub fn layout(&self) -> Arc<Ext2Layout> {
        self.layout.clone()
    }

    pub fn allocator(&self) -> Arc<Ext2Allocator> {
        self.allocator.clone()
    }

    pub fn parent_inode(&self) -> Inode {
        self.layout
            .inode_nth(self.parent_id(), self.layout(), self.allocator())
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

    fn blocks_needed(old_size: usize, new_size: usize) -> usize {
        todo!()
    }

    fn blocks_freed(old_size: usize, new_size: usize) -> usize {
        todo!()
    }

    pub fn increase_to(&self, new_size: usize) -> VfsResult<()> {
        assert!(self.size() > new_size);
        // 计算申请的 block 数,
        // 从 bitmap 得到 idx 索引向量
        // ext2_inode 扩容,

        let needed_num = Self::blocks_needed(self.size(), new_size);
        let mut needed: Vec<u32> = self.allocator.alloc_data(needed_num);
        self.modify_disk_inode(|ext2_inode| {
            ext2_inode.increase_size(new_size, needed);
        });

        todo!()
    }

    pub fn decrease_to(&self, new_size: usize) -> VfsResult<()> {
        assert!(self.size() < new_size);
        // 计算释放的 block 数,
        // 从 ext2_inode 中释放 blocks, 得到索引向量
        // 在 bitmap 中释放 idx 索引向量

        let freed_num = Self::blocks_freed(self.size(), new_size);
        let mut freed: Vec<u32> = self.allocator.dealloc_data(freed_num);
        self.modify_disk_inode(|ext2_inode| {
            ext2_inode.decrease_size(new_size, freed);
        });
        todo!()
    }
}

impl VfsInode for Inode {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> VfsResult<usize> {
        Ok(self.read_disk_inode(|ext2_inode| ext2_inode.read_at(offset, buf)))
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> VfsResult<usize> {
        todo!()
    }

    fn set_len(&mut self, len: usize) -> VfsResult<()> {
        use core::cmp::Ordering;
        match self.size().cmp(&len) {
            Ordering::Less => self.increase_to(len),
            Ordering::Equal => Ok(()),
            Ordering::Greater => self.decrease_to(len),
        }
    }

    fn metadata(&self) -> Box<dyn VfsMetadata> {
        // 有趣的是, 如果函数重名(比如这里的 metadata 和 Inode 的 metadata)
        // 并不会发生冲突, 而是结构体方法优先
        Box::new(self.metadata())
    }

    fn read_symlink(&self) -> String {
        self.read_symlink()
    }
}
