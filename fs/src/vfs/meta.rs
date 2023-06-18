pub trait VfsMetadata {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VfsFileType {
    RegularFile,
    Directory,
    CharDev,
    BlockDev,
    FIFO,
    Socket,
    SymbolicLink,
}

impl VfsFileType {
    pub fn is_file(&self) -> bool {
        self == &VfsFileType::RegularFile
    }
    pub fn is_dir(&self) -> bool {
        self == &VfsFileType::Directory
    }
    pub fn is_symlink(&self) -> bool {
        self == &VfsFileType::SymbolicLink
    }
}

#[derive(Debug, Clone)]
pub struct VfsPermissions {
    user: VfsPermission,
    group: VfsPermission,
    others: VfsPermission,
}

impl VfsPermissions {
    pub fn new(user: VfsPermission, group: VfsPermission, others: VfsPermission) -> Self {
        Self {
            user,
            group,
            others,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VfsPermission {
    read: bool,
    write: bool,
    execute: bool,
}

impl VfsPermission {
    pub fn new(read: bool, write: bool, execute: bool) -> Self {
        Self {
            read,
            write,
            execute,
        }
    }
}

impl From<u8> for VfsPermission {
    fn from(value: u8) -> Self {
        Self {
            read: (value & 0x1) != 0,
            write: (value & 0x2) != 0,
            execute: (value & 0x4) != 0,
        }
    }
}
