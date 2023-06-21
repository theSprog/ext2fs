use core::fmt::{self, Debug};

use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

use crate::{
    block::{self, DataBlock},
    block_device, cast,
};

use super::{
    address::Address, allocator::Ext2Allocator, disk_inode::Ext2Inode, inode::Inode,
    layout::Ext2Layout,
};

#[repr(C)]
#[derive(Clone)]
pub struct Ext2BlockGroupDesc {
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

const UNIT_WIDTH: usize = 64;
type BitmapBlock = [u64; block::SIZE / UNIT_WIDTH];

impl Ext2BlockGroupDesc {
    pub(crate) fn find(count: u32) -> Vec<Self> {
        block_device::read(1, 0, |data: &DataBlock| {
            let mut vec = Vec::new();
            let mut offset = 0;
            for i in 0..count {
                let current = &data[offset..];
                let desc = cast!(current.as_ptr(), Ext2BlockGroupDesc);
                vec.push(desc.clone());
                offset += core::mem::size_of::<Ext2BlockGroupDesc>();
            }
            vec
        })
    }

    fn bitmap_block_bid(&self) -> usize {
        self.block_bitmap_addr as usize
    }

    fn inode_bitmap_bid(&self) -> usize {
        self.inode_bitmap_addr as usize
    }

    fn inode_table_bid(&self) -> usize {
        self.inode_table_block as usize
    }

    pub fn get_inode(
        &self,
        inode_id: usize,
        inode_innner_idx: usize,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
    ) -> Inode {
        let address = Address::new(
            self.inode_table_bid(),
            (inode_innner_idx * core::mem::size_of::<Ext2Inode>()) as isize,
        );
        Inode::new(inode_id, address, layout, allocator)
    }

    // 调用该函数必然成功, 所有的检查应该在外部完成
    pub fn alloc_inode(&mut self, is_dir: bool) -> u32 {
        todo!()
    }

    pub fn dealloc_inode(&mut self, idx: usize, is_dir: bool) {
        todo!()
    }

    // 调用该函数必然成功, 所有的检查应该在外部完成
    // 在本 blockgroup 中尽力分配 num 个 block, 但是不一定能完成
    pub fn alloc_blocks(&mut self, num: usize) -> Vec<u32> {
        let mut vec = Vec::new();
        // 不能提前更新 free_blocks_count 因为不一定有 num 个满足
        block_device::modify(
            self.bitmap_block_bid() as usize,
            0,
            |bitmap: &mut BitmapBlock| {
                use core::ops::Not;
                for (pos, bits) in bitmap.iter_mut().enumerate() {
                    let mut neg_bits = bits.not();
                    while neg_bits != 0 {
                        let inner_pos = neg_bits.trailing_zeros() as usize;
                        *bits |= 1 << inner_pos;
                        // 不要忘记更新 free_blocks_count
                        self.free_blocks_count -= 1;
                        vec.push((pos * UNIT_WIDTH + inner_pos) as u32);

                        if vec.len() == num {
                            return vec;
                        }

                        neg_bits &= neg_bits - 1;
                    }
                }

                // num 没有完全满足
                return vec;
            },
        )
    }

    #[inline]
    fn decomposition(&self, bid: u32) -> (usize, usize) {
        let inner_bid = bid as usize % block::BITS;
        (inner_bid / UNIT_WIDTH, inner_bid % UNIT_WIDTH)
    }

    pub fn dealloc_blocks(&mut self, blocks: &[u32]) {
        if blocks.is_empty() {
            return;
        }

        // 提前批量更新 free_blocks_count
        self.free_blocks_count += blocks.len() as u16;

        block_device::modify(
            self.bitmap_block_bid() as usize,
            0,
            |bitmap: &mut BitmapBlock| {
                for bid in blocks {
                    let (pos, inner_pos) = self.decomposition(*bid);
                    assert_ne!(bitmap[pos] & (1u64 << inner_pos), 0);
                    bitmap[pos] -= 1u64 << inner_pos;
                }
            },
        );
    }
}

impl Debug for Ext2BlockGroupDesc {
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
