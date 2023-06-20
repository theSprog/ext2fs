use std::{fs::OpenOptions, sync::Arc};

use fs::{
    ext2::{Address, Ext2FileSystem},
    time::LocalTime,
    vfs::{VfsPath, VFS},
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
    let dir = vfs.read_dir("//cycle/new_dir2").unwrap();
    println!(
        "{:>5} {:>11} {:>5} {:>8} {:>5} {:>5} {:>19} {}",
        "Inode", "Permissions", "Links", "Size", "UID", "GID", "Modified Time", "Name"
    );

    for entry in dir {
        let metadata = entry.inode().metadata();
        let name = if metadata.filetype().is_symlink() {
            format!("{} -> {}", entry.name(), entry.inode().read_symlink())
        } else {
            format!("{}", entry.name())
        };

        println!(
            "{:>5}  {}{} {:>5} {:>8} {:>5} {:>5} {:>19} {}",
            entry.inode_id(),
            metadata.filetype(),
            metadata.permissions(),
            metadata.hard_links(),
            metadata.size(),
            metadata.uid(),
            metadata.gid(),
            LocalTime::from_posix(metadata.timestamp().mtime()),
            name
        );
    }
}

#[test]
fn test_syntax() {}
