use std::{fs::OpenOptions, sync::Arc};

use fs::{
    block,
    ext2::Ext2FileSystem,
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
    let dir = vfs.read_dir("/").unwrap();
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
fn test_exists() {
    let vfs = gen_vfs();
    assert_eq!(vfs.exists("/none").unwrap(), false);
    assert_eq!(vfs.exists("/new_file.c").unwrap(), true);
    assert_eq!(vfs.exists("/new_sym").unwrap(), true);
    assert_eq!(vfs.exists("/symlink").unwrap(), true);
    assert_eq!(vfs.exists("/hardlink").unwrap(), true);
    assert_eq!(vfs.exists("/new_dir").unwrap(), true);
    assert_eq!(vfs.exists("/new_dir/cycle").unwrap(), true);
}

#[test]
fn test_read_file() {
    let vfs = gen_vfs();
    let file = vfs.open_file("/new_file.c").unwrap();
    let mut buf = [0u8; 8192];
    let count = file.read_at(0, &mut buf).unwrap();
    println!("{}", core::str::from_utf8(&buf).unwrap());
    println!("count: {}", count);
}

#[test]
fn test_rw() {
    let vfs = gen_vfs();
    let mut file = vfs.open_file("/new_file.c").unwrap();

    let mut buffer = [0u8; 4096];
    let mut random_str_test = |len: usize| {
        println!("rand test: {}", len);
        file.set_len(0).unwrap();
        assert_eq!(file.read_at(0, &mut buffer).unwrap(), 0);
        let mut str = String::new();
        use rand;
        // random digit
        for _ in 0..len {
            str.push(char::from('0' as u8 + rand::random::<u8>() % 10));
        }
        file.write_at(0, str.as_bytes()).unwrap();
        let mut read_buffer = [0u8; 8192];
        let mut offset = 0usize;
        let mut read_str = String::new();
        loop {
            let len = file.read_at(offset, &mut read_buffer).unwrap();
            if len == 0 {
                break;
            }
            offset += len;
            read_str.push_str(core::str::from_utf8(&read_buffer[..len]).unwrap());
        }
        assert_eq!(str, read_str);
    };

    use rand::Rng;
    let block_size = block::SIZE;
    let mut rng = rand::thread_rng();
    for i in 0..5 {
        random_str_test(rng.gen_range(0..2300 * block_size));
        random_str_test(rng.gen_range(0..800 * block_size));
        random_str_test(rng.gen_range(0..3 * block_size));
        random_str_test(rng.gen_range(0..1800 * block_size));
        random_str_test(rng.gen_range(0..5 * block_size));
        random_str_test(rng.gen_range(0..500 * block_size));
        random_str_test(rng.gen_range(0..1500 * block_size));
    }

    vfs.flush().unwrap();
}

#[test]
fn test_syntax() {}
