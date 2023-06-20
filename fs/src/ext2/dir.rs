use core::fmt::{Debug, Display};

use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::{self, Vec},
};

use crate::{
    cast,
    vfs::{
        error::{IOError, IOErrorKind, VfsResult},
        VfsDirEntry, VfsInode, VfsPath,
    },
};

use super::{
    allocator::{self, Ext2Allocator},
    disk_inode::Ext2Inode,
    inode::Inode,
    layout::Ext2Layout,
};

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
    allocator: Arc<Ext2Allocator>,
}
impl DirEntry {
    fn new(
        inode_id: usize,
        parent_id: usize,
        name: String,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Ext2Allocator>,
    ) -> Self {
        Self {
            name,
            inode_id,
            parent_id,
            layout,
            allocator,
        }
    }
}

impl Debug for DirEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{} -> {}", self.name, self.inode_id)
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
                .inode_nth(self.inode_id, self.layout.clone(), self.allocator.clone())
                .with_parent(self.parent_id),
        )
    }
}

pub struct Dir {
    inode_id: usize,
    buffer: Vec<u8>,
    layout: Arc<Ext2Layout>,
    allocator: Arc<Ext2Allocator>,
}

impl Dir {
    pub fn from_inode(
        inode_id: usize,
        ext2_inode: &Ext2Inode,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Ext2Allocator>,
    ) -> Self {
        let mut buffer = alloc::vec![0; ext2_inode.size()];
        ext2_inode.read_at(0, &mut buffer);
        Self {
            inode_id,
            buffer,
            layout,
            allocator,
        }
    }

    fn inode_id(&self) -> usize {
        self.inode_id
    }

    pub(crate) fn entries(&self) -> Vec<DirEntry> {
        let mut entries = Vec::new();
        for (offset, entry) in self.split() {
            let entry_id = entry.inode_id as usize;
            let name = String::from_utf8(entry.name_bytes().to_vec()).unwrap();
            entries.push(DirEntry::new(
                entry_id,
                self.inode_id(),
                name,
                self.layout.clone(),
                self.allocator.clone(),
            ));
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

impl Inode {
    // 读当前 inode 下所有目录下, 如果当前 inode 不是目录抛出异常
    pub fn read_dir(&self) -> VfsResult<Vec<Box<dyn VfsDirEntry>>> {
        if !self.is_dir() {
            return Err(IOError::new(IOErrorKind::NotADirectory).into());
        }

        Ok(self
            .inner_read_dir()
            .into_iter()
            .map(|x| Box::new(x) as Box<dyn VfsDirEntry>)
            .collect())
    }

    fn inner_read_dir(&self) -> Vec<DirEntry> {
        assert!(self.is_dir());

        self.read_disk_inode(|ext2_inode| {
            let dir = Dir::from_inode(self.inode_id(), ext2_inode, self.layout(), self.allocator());
            dir.entries()
        })
    }

    // 从 path 一直走到终点, 遇到 symlink 也解析并继续走
    pub(crate) fn walk(&self, path: &VfsPath) -> VfsResult<Inode> {
        let last = self.goto_last(path)?;
        if last.is_symlink() {
            let parent_last = last.parent_inode();
            parent_last.walk(&last.symlink_target(path)?)
        } else {
            Ok(last)
        }
    }

    fn goto_last(&self, path: &VfsPath) -> VfsResult<Inode> {
        let mut current_inode = self.clone();
        let mut next_path = VfsPath::empty(path.is_from_root());
        for next in path.iter() {
            next_path.push(next);

            if current_inode.is_symlink() {
                let parent = current_inode.parent_inode();
                let symlink_path = current_inode.symlink_target(path)?;
                if symlink_path.is_from_root() {
                    let root = self.layout().root_inode(self.layout(), self.allocator());
                    current_inode = root.walk(&symlink_path)?;
                } else {
                    current_inode = parent.walk(&symlink_path)?;
                }
            }

            if !current_inode.is_dir() {
                return Err(IOError::new(IOErrorKind::NotADirectory)
                    .with_path(next_path.to_string())
                    .into());
            }

            let entries = current_inode.inner_read_dir();
            current_inode = self
                .child_inode(&entries, next)
                .map_err(|err| err.with_path(next_path.to_string()))?;
        }
        Ok(current_inode)
    }

    fn child_inode(&self, entries: &[DirEntry], next: &str) -> VfsResult<Inode> {
        let chosen = Self::find_single(entries, next);
        if chosen.is_none() {
            return Err(IOError::new(IOErrorKind::NotFound).into());
        }
        let child_id = chosen.unwrap().inode_id();
        Ok(self
            .layout()
            .inode_nth(child_id, self.layout(), self.allocator().clone())
            .with_parent(self.inode_id()))
    }

    fn find_single<'a>(entries: &'a [DirEntry], filename: &str) -> Option<&'a DirEntry> {
        let mut found_entry = None;

        for entry in entries {
            if entry.name() == filename {
                if found_entry.is_some() {
                    panic!(
                        "Multiple entries found with filename: {}, entries: {:#?}",
                        filename, entries
                    );
                }
                found_entry = Some(entry);
            }
        }

        found_entry
    }
}
