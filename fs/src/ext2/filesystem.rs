use core::fmt::{self, Display};

use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use spin::Mutex;

use crate::{
    block_device::{self, BlockDevice},
    ext2::{allocator, superblock},
};

use crate::vfs::{error::VfsResult, meta::*, VfsDirEntry, VfsInode, VfsPath};

use super::{
    allocator::Ext2Allocator, blockgroup::Ext2BlockGroupDesc, inode::Inode, layout::Ext2Layout,
    superblock::Superblock,
};

#[derive(Debug)]
pub struct Ext2FileSystem {
    layout: Arc<Ext2Layout>,
    allocator: Arc<Mutex<Ext2Allocator>>,
}

impl Display for Ext2FileSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:#?}", self.layout)
    }
}

impl Ext2FileSystem {
    pub fn open(block_dev: impl BlockDevice) -> Self {
        block_device::register_block_device(block_dev);
        let superblock = block_device::read(0, 1024, |sb: &Superblock| {
            sb.check_valid();
            sb.clone()
        });

        let blockgroup_count = superblock.blockgroup_count();
        let blockgroups = Ext2BlockGroupDesc::find(blockgroup_count);

        let layout = Arc::new(Ext2Layout::new(superblock, blockgroups));
        let allocator = Arc::new(Mutex::new(Ext2Allocator::new(layout.clone())));

        Self { layout, allocator }
    }

    pub fn flush(&self) {
        self.layout.flush();
    }

    fn root_inode(&self) -> Inode {
        self.layout
            .root_inode(self.layout.clone(), self.allocator.clone())
    }
}

use crate::vfs::FileSystem;
impl FileSystem for Ext2FileSystem {
    fn read_dir(&self, path: VfsPath) -> VfsResult<Vec<Box<dyn VfsDirEntry>>> {
        let root_inode = self.root_inode();
        let target = root_inode.walk(&path)?;
        target
            .read_dir()
            .map_err(|err| err.with_path(path.to_string()))
    }

    fn exists(&self, path: VfsPath) -> VfsResult<bool> {
        let root_inode = self.root_inode();
        let target = root_inode.walk(&path);
        Ok(target.is_ok())
    }

    fn metadata(&self, path: VfsPath) -> VfsResult<Box<dyn VfsMetadata>> {
        let root_inode = self.root_inode();
        let target = root_inode.walk(&path)?;
        Ok(Box::new(target.metadata()))
    }

    fn link(&self, to: VfsPath, from: VfsPath) -> VfsResult<()> {
        let root_inode = self.root_inode();
        // to 必须要存在
        let target = root_inode.walk(&to)?;
        let mut dir_inode = root_inode.walk(&from.parent())?;

        dir_inode.insert_hardlink(&from, &to, &target)?;
        Ok(())
    }

    fn symlink(&self, to: VfsPath, from: VfsPath) -> VfsResult<()> {
        let root_inode = self.root_inode();
        // to 可以不存在
        let mut dir_inode = root_inode.walk(&from.parent())?;

        dir_inode.insert_entry(&from, VfsFileType::SymbolicLink)?;
        Ok(())
    }

    fn open_file(&self, path: VfsPath) -> VfsResult<Box<dyn VfsInode>> {
        let root_inode = self.root_inode();
        let target = root_inode.walk(&path)?;
        Ok(Box::new(target))
    }

    fn create_file(&self, path: VfsPath) -> VfsResult<Box<dyn VfsInode>> {
        let root_inode = self.root_inode();
        let mut dir_inode = root_inode.walk(&path.parent())?;
        dir_inode.insert_entry(&path, VfsFileType::RegularFile)
    }

    fn remove_file(&self, path: VfsPath) -> VfsResult<()> {
        todo!()
    }

    fn create_dir(&self, path: VfsPath) -> VfsResult<()> {
        todo!()
    }

    fn remove_dir(&self, path: VfsPath) -> VfsResult<()> {
        todo!()
    }

    fn flush(&self) {
        self.flush();
    }
}
