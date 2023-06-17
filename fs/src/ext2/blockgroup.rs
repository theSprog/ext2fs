use core::fmt::{self, Debug};

use alloc::vec::Vec;

use crate::{block::DataBlock, block_device};

#[repr(C)]
#[derive(Clone)]
pub struct BlockGroupDescriptor {
    /// Block address of block usage bitmap
    pub block_bitmap_addr: u32,
    /// Block address of inode usage bitmap
    pub inode_bitmap_addr: u32,
    /// Starting block address of inode table
    pub inode_table_block: u32,
    /// Number of unallocated blocks in group
    pub free_blocks_count: u16,
    /// Number of unallocated inodes in group
    pub free_inodes_count: u16,
    /// Number of directories in group
    pub dirs_count: u16,
    #[doc(hidden)]
    _reserved: [u8; 14],
}
impl BlockGroupDescriptor {
    pub(crate) fn find(count: u32) -> Vec<Self> {
        block_device::read(1, 0, |data: &DataBlock| {
            let mut vec = Vec::new();
            let mut offset = 0;
            for i in 0..count {
                let current = &data[offset..];
                let desc = unsafe { &*(current.as_ptr() as *const BlockGroupDescriptor) };
                vec.push(desc.clone());
                offset += core::mem::size_of::<BlockGroupDescriptor>();
            }
            vec
        })
    }
}

impl Debug for BlockGroupDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockGroupDescriptor")
            .field("block_bitmap_addr", &self.block_bitmap_addr)
            .field("inode_bitmap_addr", &self.inode_bitmap_addr)
            .field("inode_table_block", &self.inode_table_block)
            .field("free_blocks_count", &self.free_blocks_count)
            .field("free_inodes_count", &self.free_inodes_count)
            .field("dirs_count", &self.dirs_count)
            .finish()
    }
}
