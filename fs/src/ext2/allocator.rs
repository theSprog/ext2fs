use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

use super::{blockgroup::Ext2BlockGroupDesc, layout::Ext2Layout, superblock::Superblock};

#[derive(Debug)]
pub struct Ext2Allocator {
    blocks_per_group: u32,
    inodes_per_group: u32,

    superblock: Arc<Mutex<Superblock>>,
    blockgroups: Arc<Vec<Mutex<Ext2BlockGroupDesc>>>,
}
impl Ext2Allocator {
    pub(crate) fn new(layout: Arc<Ext2Layout>) -> Ext2Allocator {
        Self {
            blocks_per_group: layout.blocks_per_group(),
            inodes_per_group: layout.inodes_per_group(),
            superblock: layout.superblock(),
            blockgroups: layout.blockgroups(),
        }
    }

    pub(crate) fn alloc_data(&self, needed: usize) -> Vec<u32> {
        todo!()
    }

    pub(crate) fn dealloc_data(&self, needed: usize) -> Vec<u32> {
        todo!()
    }
}
