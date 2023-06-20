use core::fmt::Debug;

use alloc::{boxed::Box, string::String};

use super::meta::VfsMetadata;

pub trait VfsInode: Debug {
    fn metadata(&self) -> Box<dyn VfsMetadata> {
        unimplemented!()
    }

    fn read_symlink(&self) -> String;
}
