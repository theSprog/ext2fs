use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

use super::{
    blockgroup::{self, Ext2BlockGroupDesc},
    inode::Inode,
    superblock::{self, Superblock},
};

#[derive(Debug)]
pub struct Ext2Layout {
    blocks_per_group: u32,
    inodes_per_group: u32,

    superblock: Arc<Mutex<Superblock>>,
    blockgroups: Arc<Vec<Mutex<Ext2BlockGroupDesc>>>,
}

impl Ext2Layout {
    pub fn new(superblock: Superblock, blockgroups: Vec<Ext2BlockGroupDesc>) -> Self {
        let blocks_per_group = superblock.blocks_per_group;
        let inodes_per_group = superblock.inodes_per_group;

        let superblock = Arc::new(Mutex::new(superblock));
        // 为每一个成员加上锁
        let blockgroups = Arc::new(blockgroups.into_iter().map(Mutex::new).collect::<Vec<_>>());

        Self {
            blocks_per_group,
            inodes_per_group,
            superblock,
            blockgroups,
        }
    }

    pub fn superblock(&self) -> Arc<Mutex<Superblock>> {
        self.superblock.clone()
    }

    pub fn blockgroups(&self) -> Arc<Vec<Mutex<Ext2BlockGroupDesc>>> {
        self.blockgroups.clone()
    }

    pub fn blocks_per_group(&self) -> u32 {
        self.blocks_per_group
    }
    pub fn inodes_per_group(&self) -> u32 {
        self.inodes_per_group
    }

    pub fn root_inode(&self, layout: Arc<Ext2Layout>) -> Inode {
        self.inode_nth(2, layout).with_parent(2)
    }

    pub fn inode_nth(&self, inode_id: usize, layout: Arc<Ext2Layout>) -> Inode {
        // 拿到所在 block_group 和 inode 内部偏移量
        let (blockgroup_idx, inode_innner_idx) = self.inode_idx(inode_id);
        let bg = self.blockgroups.get(blockgroup_idx).unwrap().lock();
        bg.get_inode(inode_id, inode_innner_idx).with_layout(layout)
    }

    fn inode_idx(&self, inode_id: usize) -> (usize, usize) {
        let inode_seq: usize = inode_id - 1;
        let blockgroup_idx = inode_seq / self.inodes_per_group as usize;
        let inode_innner_idx = inode_seq % self.inodes_per_group as usize;
        (blockgroup_idx, inode_innner_idx)
    }
}
