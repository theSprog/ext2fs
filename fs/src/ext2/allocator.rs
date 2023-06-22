use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

use crate::vfs::error::{IOError, IOErrorKind, VfsResult};

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

    fn free_blocks(&self) -> u32 {
        let sb = self.superblock.lock();
        sb.free_blocks_count - sb.r_blocks_count
    }

    fn inc_free_blocks(&mut self, n: usize) {
        self.superblock.lock().free_blocks_count += n as u32;
    }

    fn dec_free_blocks(&mut self, n: usize) {
        self.superblock.lock().free_blocks_count -= n as u32;
    }

    fn inc_free_inode(&mut self) {
        self.superblock.lock().free_inodes_count += 1;
    }

    fn dec_free_inode(&mut self) {
        self.superblock.lock().free_inodes_count -= 1;
    }

    fn free_inodes(&self) -> u32 {
        self.superblock.lock().free_inodes_count
    }

    pub(crate) fn alloc_inode(&mut self, is_dir: bool) -> VfsResult<u32> {
        if self.free_inodes() == 0 {
            return Err(IOError::new(IOErrorKind::NoFreeInodes).into());
        }

        // 有可用 inode
        self.dec_free_inode();
        for bg in self.blockgroups.iter() {
            let mut bg: spin::MutexGuard<'_, Ext2BlockGroupDesc> = bg.lock();
            if bg.free_blocks_count == 0 {
                continue;
            }
            return Ok(bg.alloc_inode(is_dir));
        }

        unreachable!()
    }
    pub(crate) fn dealloc_inode(&self, block_id: usize, is_dir: bool) -> VfsResult<()> {
        todo!()
    }

    pub(crate) fn alloc_data(&mut self, needed: usize) -> VfsResult<Vec<u32>> {
        if needed > self.free_blocks() as usize {
            return Err(IOError::new(IOErrorKind::NoFreeBlocks).into());
        }

        let mut unmet = needed;
        let mut ret = Vec::new();
        // 需要分别更新 superblock 的 free_blocks 和 blockgroups 的 free_blocks_count
        for bg in self.blockgroups.iter() {
            let mut bg: spin::MutexGuard<'_, Ext2BlockGroupDesc> = bg.lock();
            // 每一个 bg 都尽力分配 unmet 个块, 返回分配的块数
            let allocated = bg.alloc_blocks(unmet);
            unmet -= allocated.len();
            ret.extend(allocated);
            if unmet == 0 {
                break;
            }
        }

        // 扣除 free_blocks
        self.dec_free_blocks(needed);
        // 前面判断有空间, 因此跳出循环时必然 unmet == 0
        assert_eq!(unmet, 0);
        Ok(ret)
    }

    pub(crate) fn dealloc_data(&mut self, mut freed: Vec<u32>) -> VfsResult<()> {
        let mut slots = alloc::vec![0; self.blockgroups.len()];

        // 让所有同一 blockgroup 的聚集在连续一块
        freed.sort();
        // 标出分别属于哪一个 block
        for idx in &freed {
            let bg_idx = idx / self.blocks_per_group;
            slots[bg_idx as usize] += 1;
        }

        let mut offset = 0;
        for (idx, bg) in self.blockgroups.iter().enumerate() {
            let mut bg = bg.lock();
            bg.dealloc_blocks(&freed[offset..offset + slots[idx]]);
            offset += slots[idx];
        }

        self.inc_free_blocks(freed.len());
        assert_eq!(offset, freed.len());

        Ok(())
    }
}
