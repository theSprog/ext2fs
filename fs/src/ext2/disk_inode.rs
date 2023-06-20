use bitflags::bitflags;

use crate::{
    block::{self, DataBlock},
    block_device,
    vfs::meta::*,
};

#[repr(C)]
#[derive(Clone)]
pub struct Ext2Inode {
    /// Type and Permissions (see below)
    pub type_perm: TypePerm,
    /// User ID
    pub uid: u16,
    /// Lower 32 bits of size in bytes
    pub size_low: u32,
    /// Last Access Time (in POSIX time)
    pub atime: u32,
    /// Creation Time (in POSIX time)
    pub ctime: u32,
    /// Last Modification time (in POSIX time)
    pub mtime: u32,
    /// Deletion time (in POSIX time)
    pub dtime: u32,
    /// Group ID
    pub gid: u16,
    /// Count of hard links (directory entries) to this inode. When this
    /// reaches 0, the data blocks are marked as unallocated.
    pub hard_links: u16,
    /// Count of disk sectors (not Ext2 blocks) in use by this inode, not
    /// counting the actual inode structure nor directory entries linking
    /// to the inode.
    pub sectors_count: u32,
    /// Flags
    pub flags: Flags,
    /// Operating System Specific value #1
    pub _os_specific_1: [u8; 4],
    /// Direct block pointers
    pub direct_pointer: [u32; 12],
    /// Singly Indirect Block Pointer (Points to a block that is a list of
    /// block pointers to data)
    pub indirect_pointer: u32,
    /// Doubly Indirect Block Pointer (Points to a block that is a list of
    /// block pointers to Singly Indirect Blocks)
    pub doubly_indirect: u32,
    /// Triply Indirect Block Pointer (Points to a block that is a list of
    /// block pointers to Doubly Indirect Blocks)
    pub triply_indirect: u32,
    /// Generation number (Primarily used for NFS)
    pub gen_number: u32,
    /// In Ext2 version 0, this field is reserved. In version >= 1,
    /// Extended attribute block (File ACL).
    pub ext_attribute_block: u32,
    /// In Ext2 version 0, this field is reserved. In version >= 1, Upper
    /// 32 bits of file size (if feature bit set) if it's a file,
    /// Directory ACL if it's a directory
    pub size_high: u32,
    /// Block address of fragment
    pub frag_block_addr: u32,
    /// Operating System Specific Value #2
    pub _os_specific_2: [u8; 12],
}

type IndirectBlock = [u32; Ext2Inode::INDIRECT_COUNT];

impl Ext2Inode {
    pub const DIRECT_COUNT: usize = 12;
    pub const INDIRECT_COUNT: usize = block::SIZE / 4;
    pub const INDIRECT_BOUND: usize = Self::DIRECT_COUNT + Self::INDIRECT_COUNT;
    pub const DOUBLE_COUNT: usize = Self::INDIRECT_COUNT * Self::INDIRECT_COUNT;
    pub const DOUBLE_BOUND: usize = Self::INDIRECT_BOUND + Self::DOUBLE_COUNT;

    pub fn init() {
        todo!()
    }

    pub fn filetype(&self) -> VfsFileType {
        self.type_perm.filetype()
    }

    pub fn permissions(&self) -> VfsPermissions {
        self.type_perm.permissions()
    }

    pub fn size(&self) -> usize {
        if self.filetype().is_file() {
            assert_eq!(self.size_high, 0);
        }
        self.size_low as usize
    }

    pub fn timestamp(&self) -> VfsTimeStamp {
        VfsTimeStamp::new(
            self.atime as u64,
            self.ctime as u64,
            self.mtime as u64,
            self.dtime as u64,
        )
    }

    pub fn uid(&self) -> u16 {
        self.uid
    }
    pub fn gid(&self) -> u16 {
        self.gid
    }

    pub fn hard_links(&self) -> u16 {
        self.hard_links
    }

    fn block_nth(&self, inner_idx: u32) -> u32 {
        let inner_idx = inner_idx as usize;
        if inner_idx < Self::DIRECT_COUNT {
            self.direct_pointer[inner_idx]
        } else if inner_idx < Self::INDIRECT_BOUND {
            block_device::read(
                self.indirect_pointer as usize,
                0,
                |indirect_block: &IndirectBlock| indirect_block[inner_idx - Self::DIRECT_COUNT],
            )
        } else if inner_idx < Self::DOUBLE_BOUND {
            let last = inner_idx - Self::INDIRECT_BOUND;
            let indirect = block_device::read(
                self.doubly_indirect as usize,
                0,
                |indirect2: &IndirectBlock| indirect2[last / Self::INDIRECT_COUNT],
            );

            block_device::read(indirect as usize, 0, |indirect1: &IndirectBlock| {
                indirect1[last % Self::INDIRECT_COUNT]
            })
        } else {
            panic!("where is the large block from : inner_id = {}", inner_idx);
        }
    }

    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let block_size = block::SIZE;
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size());
        if start >= end {
            return 0;
        }
        let mut start_block = start / block_size;
        let mut read_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block = (start / block_size + 1) * block_size;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let block_read_size = end_current_block - start;
            let dst = &mut buf[read_size..read_size + block_read_size];

            block_device::read(
                self.block_nth(start_block as u32) as usize,
                0,
                |data_block: &DataBlock| {
                    let src = &data_block[start % block_size..start % block_size + block_read_size];
                    dst.copy_from_slice(src);
                },
            );

            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        read_size
    }
}

bitflags! {
    #[derive(Clone)]
    pub struct TypePerm: u16 {
        /// FIFO
        const FIFO = 0x1000;
        /// Character device
        const CHAR_DEVICE = 0x2000;
        /// Directory
        const DIRECTORY = 0x4000;
        /// Block device
        const BLOCK_DEVICE = 0x6000;
        /// Regular file
        const FILE = 0x8000;
        /// Symbolic link
        const SYMLINK = 0xA000;
        /// Unix socket
        const SOCKET = 0xC000;
        /// Other—execute permission
        const O_EXEC = 0x001;
        /// Other—write permission
        const O_WRITE = 0x002;
        /// Other—read permission
        const O_READ = 0x004;
        /// Group—execute permission
        const G_EXEC = 0x008;
        /// Group—write permission
        const G_WRITE = 0x010;
        /// Group—read permission
        const G_READ = 0x020;
        /// User—execute permission
        const U_EXEC = 0x040;
        /// User—write permission
        const U_WRITE = 0x080;
        /// User—read permission
        const U_READ = 0x100;
        /// Sticky Bit
        const STICKY = 0x200;
        /// Set group ID
        const SET_GID = 0x400;
        /// Set user ID
        const SET_UID = 0x800;
    }
}

impl TypePerm {
    pub fn filetype(&self) -> VfsFileType {
        match self {
            _ if self.contains(Self::SOCKET) => VfsFileType::Socket,
            _ if self.contains(Self::SYMLINK) => VfsFileType::SymbolicLink,
            _ if self.contains(Self::FILE) => VfsFileType::RegularFile,
            _ if self.contains(Self::BLOCK_DEVICE) => VfsFileType::BlockDev,
            _ if self.contains(Self::DIRECTORY) => VfsFileType::Directory,
            _ if self.contains(Self::CHAR_DEVICE) => VfsFileType::CharDev,
            _ if self.contains(Self::FIFO) => VfsFileType::FIFO,
            _ => unreachable!(),
        }
    }

    pub fn permissions(&self) -> VfsPermissions {
        let mut user = 0u8;
        let mut group = 0u8;
        let mut other = 0u8;
        if self.contains(Self::U_READ) {
            user |= 0b100;
        }
        if self.contains(Self::U_WRITE) {
            user |= 0b010;
        }
        if self.contains(Self::U_EXEC) {
            user |= 0b001;
        }
        if self.contains(Self::G_READ) {
            group |= 0b100;
        }
        if self.contains(Self::G_WRITE) {
            group |= 0b010;
        }
        if self.contains(Self::G_EXEC) {
            group |= 0b001;
        }
        if self.contains(Self::O_READ) {
            other |= 0b100;
        }
        if self.contains(Self::O_WRITE) {
            other |= 0b010;
        }
        if self.contains(Self::O_EXEC) {
            other |= 0b001;
        }
        VfsPermissions::new(user.into(), group.into(), other.into())
    }
}

bitflags! {
    #[derive(Clone)]
    pub struct Flags: u32 {
        /// Secure deletion (not used)
        const SECURE_DEL = 0x00000001;
        /// Keep a copy of data when deleted (not used)
        const KEEP_COPY = 0x00000002;
        /// File compression (not used)
        const COMPRESSION = 0x00000004;
        /// Synchronous updates—new data is written immediately to disk
        const SYNC_UPDATE = 0x00000008;
        /// Immutable file (content cannot be changed)
        const IMMUTABLE = 0x00000010;
        /// Append only
        const APPEND_ONLY = 0x00000020;
        /// File is not included in 'dump' command
        const NODUMP = 0x00000040;
        /// Last accessed time should not updated
        const DONT_ATIME = 0x00000080;
        /// Hash indexed directory
        const HASH_DIR = 0x00010000;
        /// AFS directory
        const AFS_DIR = 0x00020000;
        /// Journal file data
        const JOURNAL_DATA = 0x00040000;
    }
}
