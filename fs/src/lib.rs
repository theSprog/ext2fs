#![no_std]
#![feature(error_in_core)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate alloc;

pub mod block_device;
pub mod ext2;
pub mod time;
pub mod vfs;

mod util;

const SECTOR_SIZE: usize = 512;

pub mod block {
    use super::SECTOR_SIZE;
    pub const SIZE: usize = 4096;
    pub const LOG_SIZE: usize = 12;
    pub const BITS: usize = SIZE * 8;
    pub const MASK: usize = SIZE - 1;
    pub const SECTORS_PER_BLOCK: usize = SIZE / SECTOR_SIZE;

    pub type DataBlock = [u8; SIZE];
    pub type BitmapBlock = [u64; SIZE / 64];
}

use crate::block_device::BlockCacheManager;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> =
        Mutex::new(BlockCacheManager::default());
}
