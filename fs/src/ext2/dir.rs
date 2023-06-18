use alloc::{
    boxed::Box,
    string::String,
    sync::Arc,
    vec::{self, Vec},
};

use crate::{
    cast,
    vfs::{VfsDirEntry, VfsInode},
};

use super::{disk_inode::Ext2Inode, layout::Ext2Layout};

#[repr(C)]
#[derive(Clone)]
pub struct Ext2DirEntry {
    inode_id: u32,
    record_len: u16,
    name_len: u8,
    filetype: u8,
    name: u8,
}

impl Ext2DirEntry {
    // 去掉末尾的 name 留下的长度, 有了它就可用从结构体头偏移到 name 起始处
    const BARE_LEN: usize = 8;

    pub fn name_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const _ as *const u8).add(Self::BARE_LEN),
                self.name_len as usize,
            )
        }
    }

    pub fn name_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (self as *mut _ as *mut u8).add(Self::BARE_LEN),
                self.name_len as usize,
            )
        }
    }
}

pub struct DirEntry {
    name: String,
    inode_id: usize,
    parent_id: usize,
    layout: Arc<Ext2Layout>,
}
impl DirEntry {
    fn new(inode_id: usize, parent_id: usize, name: String, layout: Arc<Ext2Layout>) -> Self {
        Self {
            name,
            inode_id,
            parent_id,
            layout,
        }
    }
}

impl VfsDirEntry for DirEntry {
    fn inode_id(&self) -> usize {
        self.inode_id
    }
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn inode(&self) -> Box<dyn VfsInode> {
        Box::new(
            self.layout
                .inode_nth(self.inode_id, self.layout.clone())
                .with_parent(self.parent_id),
        )
    }
}

pub struct Dir {
    self_id: usize,
    buffer: Vec<u8>,
    layout: Arc<Ext2Layout>,
}

impl Dir {
    pub fn from_inode(self_id: usize, ext2_inode: &Ext2Inode, layout: Arc<Ext2Layout>) -> Self {
        let mut buffer = alloc::vec![0; ext2_inode.size()];
        ext2_inode.read_at(0, &mut buffer);
        Self {
            self_id,
            buffer,
            layout,
        }
    }

    fn inode_id(&self) -> usize {
        self.self_id
    }

    pub(crate) fn entries(&self) -> Vec<Box<DirEntry>> {
        let mut entries = Vec::new();
        for (offset, entry) in self.split() {
            let inode_id = entry.inode_id as usize;
            let name = String::from_utf8(entry.name_bytes().to_vec()).unwrap();
            entries.push(Box::new(DirEntry::new(
                inode_id,
                self.inode_id(),
                name,
                self.layout.clone(),
            )));
        }
        entries
    }

    fn split(&self) -> Vec<(usize, &Ext2DirEntry)> {
        let mut offset = 0;
        let mut slice = Vec::new();
        while offset < self.buffer.len() {
            let entry = cast!(self.buffer.as_ptr().add(offset), Ext2DirEntry);
            slice.push((offset, entry));
            offset += entry.record_len as usize;
        }
        slice
    }
}
