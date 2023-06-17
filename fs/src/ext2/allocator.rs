use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

use super::{blockgroup::BlockGroupDescriptor, superblock::Superblock};

#[derive(Debug)]
pub struct Ext2Allocator {
    superblock: Arc<Mutex<Superblock>>,
    blockgroups: Arc<Vec<Mutex<BlockGroupDescriptor>>>,
}
impl Ext2Allocator {
    pub(crate) fn new(
        superblock: Arc<Mutex<Superblock>>,
        blockgroups: Arc<Vec<Mutex<BlockGroupDescriptor>>>,
    ) -> Ext2Allocator {
        Self {
            superblock,
            blockgroups,
        }
    }
}
