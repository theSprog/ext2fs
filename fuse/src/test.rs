use std::{fs::OpenOptions, sync::Arc};

use fs::{ext2::Ext2FileSystem, vfs::VFS};
use spin::Mutex;

use crate::BlockFile;

fn gen_vfs() -> VFS {
    let block_file = BlockFile::create("ext2.img");
    let ext2 = Ext2FileSystem::open(block_file);
    VFS::new(ext2)
}

#[test]
fn test_vfs() {
    let vfs = gen_vfs();
    println!("{:#}", vfs);
}

#[test]
fn test_read_dir() {
    let vfs = gen_vfs();
    let dir = vfs.read_dir("/home/mrfan/rust/ext2/fuse").unwrap();
}

#[test]
fn test_syntax() {
    let last_mnt_path: &[u8] = &[
        47, 104, 111, 109, 101, 47, 109, 114, 102, 97, 110, 47, 114, 117, 115, 116, 47, 101, 120,
        116, 50, 47, 102, 117, 115, 101, 47, 109, 110,
    ];

    let str_slice = std::str::from_utf8(last_mnt_path).unwrap();
    let str_ref = str_slice.trim_end_matches(char::from(0));

    println!("{}", str_ref);
}
