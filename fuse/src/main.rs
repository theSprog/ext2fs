#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use fs::block;
use fs::block_device::BlockDevice;
use spin::Mutex;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
};

struct BlockFile(Mutex<File>);

impl BlockFile {
    fn create(filename: &str) -> Self {
        BlockFile(Mutex::new({
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(filename)
                .unwrap()
        }))
    }
}

const SECTOR_SIZE: usize = 512;

impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock();
        file.seek(SeekFrom::Start((block_id * SECTOR_SIZE) as u64))
            .expect("Error when seeking!");
        assert_eq!(
            file.read(buf).unwrap(),
            SECTOR_SIZE,
            "Not a complete block!"
        );
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock();
        file.seek(SeekFrom::Start((block_id * SECTOR_SIZE) as u64))
            .expect("Error when seeking!");
        assert_eq!(
            file.write(buf).unwrap(),
            SECTOR_SIZE,
            "Not a complete block!"
        );
    }
}

mod test;

fn main() {
    todo!()
}
