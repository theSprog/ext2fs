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
    allocator: Arc<Ext2Allocator>,
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
        let allocator = Arc::new(Ext2Allocator::new(layout.clone()));

        Self { layout, allocator }
    }

    fn root_inode(&self) -> Inode {
        self.layout.root_inode(self.layout.clone())
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
        todo!()
    }

    fn metadata(&self, path: VfsPath) -> VfsResult<alloc::boxed::Box<dyn VfsMetadata>> {
        todo!()
    }

    fn open_file(&self, path: VfsPath) -> VfsResult<alloc::boxed::Box<dyn VfsInode>> {
        todo!()
    }

    fn create_file(&self, path: VfsPath) -> VfsResult<alloc::boxed::Box<dyn VfsInode>> {
        todo!()
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
}
