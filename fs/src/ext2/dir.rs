use core::fmt::{Debug, Display};

use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::{self, Vec},
};
use spin::Mutex;

use crate::{
    cast, cast_mut, ceil,
    vfs::{
        error::{IOError, IOErrorKind, VfsErrorKind, VfsResult},
        meta::VfsFileType,
        VfsDirEntry, VfsInode, VfsPath,
    },
};

use super::{
    allocator::{self, Ext2Allocator},
    disk_inode::{Ext2Inode, TypePerm},
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
    pub const EXT2_FT_UNKNOWN: u8 = 0;
    pub const EXT2_FT_REG_FILE: u8 = 1;
    pub const EXT2_FT_DIR: u8 = 2;
    pub const EXT2_FT_CHRDEV: u8 = 3;
    pub const EXT2_FT_BLKDEV: u8 = 4;
    pub const EXT2_FT_FIFO: u8 = 5;
    pub const EXT2_FT_SOCK: u8 = 6;
    pub const EXT2_FT_SYMLINK: u8 = 7;

    pub const MAX_FILE_NAME: usize = u8::MAX as usize;
    // 去掉末尾的 name 留下的长度, 有了它就可用从结构体头偏移到 name 起始处
    const BARE_LEN: usize = 8;

    pub fn build_raw<'a>(
        buffer: &'a mut [u8],
        filename: &str,
        inode_id: usize,
        filetype: VfsFileType,
    ) -> &'a mut Self {
        let entry = cast_mut!(buffer.as_ptr(), Self);

        entry.inode_id = inode_id as u32;
        entry.name_len = filename.len() as u8;
        entry.record_len = ceil!(Self::BARE_LEN + entry.name_len as usize, 4) as u16;
        entry.filetype = match filetype {
            VfsFileType::RegularFile => Self::EXT2_FT_REG_FILE,
            VfsFileType::Directory => Self::EXT2_FT_DIR,
            VfsFileType::CharDev => Self::EXT2_FT_CHRDEV,
            VfsFileType::BlockDev => Self::EXT2_FT_BLKDEV,
            VfsFileType::FIFO => Self::EXT2_FT_FIFO,
            VfsFileType::Socket => Self::EXT2_FT_SOCK,
            VfsFileType::SymbolicLink => Self::EXT2_FT_SYMLINK,
        };

        let name_slice = &mut buffer[Self::BARE_LEN..Self::BARE_LEN + filename.len()];
        name_slice.copy_from_slice(filename.as_bytes());

        entry
    }

    // record 理论所占空间
    pub fn regular_len(&self) -> usize {
        // 4 字节对齐
        ceil!(Self::BARE_LEN + self.name_len as usize, 4)
    }

    // record 实际所占空间
    pub fn record_len(&self) -> usize {
        assert_eq!(0, self.record_len % 4);
        self.record_len as usize
    }

    pub fn has_free(&self, needed: usize) -> bool {
        // record_len 至少和 regular_len 一样大
        (self.record_len() - self.regular_len()) >= needed
    }

    // 缩小该 record 所占空间, 返回 (期望空间, 释放空间)
    pub fn rec_narrow(&mut self) -> (usize, usize) {
        let old_len = self.record_len();
        self.record_len = self.regular_len() as u16;
        (self.record_len(), old_len - self.record_len())
    }

    pub fn rec_expand(&mut self, new_len: usize) -> usize {
        let old_len = self.record_len();
        assert!(old_len <= new_len);
        self.record_len = new_len as u16;
        old_len
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, self.regular_len() as usize)
        }
    }

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
    allocator: Arc<Mutex<Ext2Allocator>>,
}
impl DirEntry {
    fn new(
        inode_id: usize,
        parent_id: usize,
        name: String,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
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
    allocator: Arc<Mutex<Ext2Allocator>>,
}

impl Dir {
    pub fn from_inode(
        inode_id: usize,
        ext2_inode: &Ext2Inode,
        layout: Arc<Ext2Layout>,
        allocator: Arc<Mutex<Ext2Allocator>>,
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

    pub fn write_to_disk(&self, ext2_inode: &mut Ext2Inode) {
        ext2_inode.write_at(0, &self.buffer);
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
        self.split_mut()
            .into_iter()
            .map(|(index, entry)| (index, entry as &Ext2DirEntry))
            .collect()
    }

    fn split_mut(&self) -> Vec<(usize, &mut Ext2DirEntry)> {
        let mut offset = 0;
        let mut slice = Vec::new();
        while offset < self.buffer.len() {
            let entry = cast_mut!(self.buffer.as_ptr().add(offset), Ext2DirEntry);
            let rec_len = entry.record_len as usize;
            slice.push((offset, entry));
            offset += rec_len;
        }
        slice
    }

    fn place_entry(&mut self, offset: usize, entry: &Ext2DirEntry) {
        let dst = &mut self.buffer[offset..offset + entry.regular_len()];
        let src = entry.as_bytes();
        dst.copy_from_slice(src);
    }

    fn insert_entry(&mut self, filename: &str, inode_id: usize, file_type: VfsFileType) {
        let mut buffer = [0u8; 4096];
        let new_entry = Ext2DirEntry::build_raw(&mut buffer, filename, inode_id, file_type);
        for (offset, entry) in self.split_mut() {
            if entry.has_free(new_entry.regular_len()) {
                let (new_len, freed) = entry.rec_narrow();
                new_entry.rec_expand(freed);
                self.place_entry(offset + new_len, &new_entry);
                break;
            }
        }
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
                    .with_path(&next_path)
                    .into());
            }

            let entries = current_inode.inner_read_dir();
            current_inode = self
                .child_inode(&entries, next)
                .map_err(|err| err.with_path(&next_path))?;
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
            .inode_nth(child_id, self.layout(), self.allocator())
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

    fn check_valid_insert(&self, path: &VfsPath) -> VfsResult<()> {
        if !self.is_dir() {
            return Err(IOError::new(IOErrorKind::NotADirectory)
                .with_path(path)
                .into());
        }

        let filename = path.last();
        if filename.is_none() {
            return Err(VfsErrorKind::InvalidPath(path.to_string()).into());
        }

        let filename = filename.unwrap();
        let entries = self.inner_read_dir();
        let chosen = Self::find_single(&entries, filename);
        if chosen.is_some() {
            return Err(IOError::new(IOErrorKind::AlreadyExists)
                .with_path(path)
                .into());
        }

        if filename.len() > Ext2DirEntry::MAX_FILE_NAME {
            return Err(IOError::new(IOErrorKind::TooLongFileName)
                .with_path(path)
                .into());
        }

        Ok(())
    }

    // 该函数不会设置权限
    pub fn insert_entry(
        &mut self,
        path: &VfsPath,
        filetype: VfsFileType,
    ) -> VfsResult<Box<dyn VfsInode>> {
        self.check_valid_insert(path)?;
        let filename = path.last().unwrap();

        match filetype {
            VfsFileType::RegularFile => self.insert_file_entry(filename),
            VfsFileType::Directory => self.insert_dir_entry(filename),
            // VfsFileType::SymbolicLink => self.insert_symlink_entry(filename),
            _ => todo!("why got {}", filetype),
        }
    }

    // hardlink 不会申请 inode
    pub fn insert_hardlink(
        &mut self,
        path_from: &VfsPath,
        path_to: &VfsPath,
        target_inode: &Inode,
    ) -> VfsResult<()> {
        self.check_valid_insert(path_from)?;

        // 除了通用检查外, 硬链接只针对 file
        if !target_inode.is_file() {
            return Err(IOError::new(IOErrorKind::NotAFile)
                .with_path(path_to)
                .into());
        }

        let filename = path_from.last().unwrap();
        self.insert_hardlink_entry(filename, target_inode)
    }

    /// 1. 申请一个 Inode
    /// 2. 在目录中创建一个目录项
    fn insert_file_entry(&mut self, filename: &str) -> VfsResult<Box<dyn VfsInode>> {
        let inode_id = self.allocator().lock().alloc_inode(false)? as usize;
        let inode = self.layout().new_inode_nth(
            inode_id,
            VfsFileType::RegularFile,
            self.layout(),
            self.allocator(),
        );

        self.modify_disk_inode(|ext2_inode| {
            let mut dir =
                Dir::from_inode(self.inode_id(), ext2_inode, self.layout(), self.allocator());
            // 建立 filename -> inode_id 的映射关系
            dir.insert_entry(filename, inode_id, VfsFileType::RegularFile);
            // dir 仅仅是内存中的数据结构, 因此需要写回磁盘
            dir.write_to_disk(ext2_inode);
        });

        Ok(Box::new(inode))
    }

    fn insert_hardlink_entry(&mut self, filename: &str, target_inode: &Inode) -> VfsResult<()> {
        // 目录下插入新目录项
        self.modify_disk_inode(|ext2_inode| {
            let mut dir =
                Dir::from_inode(self.inode_id(), ext2_inode, self.layout(), self.allocator());
            // 建立 filename -> inode_id 的映射关系
            dir.insert_entry(filename, target_inode.inode_id(), target_inode.filetype());
            // dir 仅仅是内存中的数据结构, 因此需要写回磁盘
            dir.write_to_disk(ext2_inode);
        });

        // 目标 inode 硬链接增加
        target_inode.modify_disk_inode(|ext2_inode| {
            ext2_inode.inc_hard_links();
        });
        Ok(())
    }

    /// 1. 申请一个 Inode
    /// 2. 在 dirname 下新建两个目录项, 分别是 . 和 .., 注意硬链接变化
    /// 3. 在目录中创建一个目录项
    fn insert_dir_entry(&mut self, dirname: &str) -> VfsResult<Box<dyn VfsInode>> {
        let inode_id = self.allocator().lock().alloc_inode(true)? as usize;
        let inode = self
            .layout()
            .inode_nth(inode_id, self.layout(), self.allocator());

        // self.modify_disk_inode(|ext2_inode| {
        //     let mut dir =
        //         Dir::from_inode(self.inode_id(), ext2_inode, self.layout(), self.allocator());
        //     // 建立 filename -> inode_id 的映射关系
        //     dir.insert_entry(dirname, inode_id)
        // });

        Ok(Box::new(inode))
    }
}
