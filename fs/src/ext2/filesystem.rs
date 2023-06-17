use core::fmt::{self, Display};

use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use spin::Mutex;

use crate::{
    block_device::{self, BlockDevice},
    ext2::{allocator, superblock},
    vfs::{error::VfsResult, VfsDirEntry, VfsInode, VfsMetadata, VfsPath},
};

use super::{allocator::Ext2Allocator, blockgroup::BlockGroupDescriptor, superblock::Superblock};

#[derive(Debug)]
pub struct Ext2FileSystem {
    superblock: Arc<Mutex<Superblock>>,
    blockgroups: Arc<Vec<Mutex<BlockGroupDescriptor>>>,

    allocator: Ext2Allocator,
}

impl Display for Ext2FileSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:#?}", self.superblock)?;
        writeln!(f, "{:#?}", self.blockgroups)
    }
}

impl Ext2FileSystem {
    pub fn open(block_dev: impl BlockDevice) -> Self {
        block_device::register_block_device(block_dev);
        let superblock = block_device::read(0, 1024, |sb: &Superblock| {
            sb.check_valid();
            sb.clone()
        });

        let block_group_count = superblock.block_group_count();
        let blockgroups = BlockGroupDescriptor::find(block_group_count);

        let superblock = Arc::new(Mutex::new(superblock));
        let blockgroups = Arc::new(
            blockgroups
                .into_iter()
                .map(|bg| Mutex::new(bg))
                .collect::<Vec<_>>(),
        );

        let allocator = Ext2Allocator::new(superblock.clone(), blockgroups.clone());

        Self {
            superblock,
            blockgroups,
            allocator,
        }
    }
}

use crate::vfs::FileSystem;
impl FileSystem for Ext2FileSystem {
    fn read_dir(&self, path: VfsPath) -> VfsResult<Vec<Box<dyn VfsDirEntry>>> {
        panic!("{:?}", path);
        todo!()
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
