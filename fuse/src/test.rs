use std::{fs::OpenOptions, sync::Arc};

use fs::{
    ext2::{Address, Ext2FileSystem},
    vfs::VFS,
};
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
    let dir = vfs.read_dir("/").unwrap();
    for entry in dir {
        println!(
            "{:?} {:?} {:?}",
            entry.inode_id(),
            entry.name(),
            entry.inode()
        );
    }
}

#[test]
fn test_syntax() {}
