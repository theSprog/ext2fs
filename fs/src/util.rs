use alloc::string::String;

// 无论末尾是否带 '\0' 都可以用该函数把 c 风格字符串转为 rust 风格
pub fn bytes_to_str(bytes: &[u8]) -> &str {
    let str_slice = core::str::from_utf8(bytes).unwrap();
    str_slice.trim_end_matches(char::from(0))
}

pub fn uuid_str(uuid_slice: &[u8]) -> String {
    assert_eq!(uuid_slice.len(), 16, "Input slice must have length 16");

    let mut uuid_str = String::with_capacity(36);

    for (i, byte) in uuid_slice.iter().enumerate() {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            uuid_str.push('-');
        }

        let hex = alloc::format!("{:02x}", byte);
        uuid_str.push_str(&hex);
    }

    uuid_str
}

#[macro_export]
macro_rules! cast {
    ($addr:expr, $T:ty) => {
        unsafe { &*($addr as *const $T) }
    };
}

#[macro_export]
macro_rules! cast_mut {
    ($addr:expr, $T:ty) => {
        unsafe { &mut *($addr as *mut $T) }
    };
}

#[macro_export]
macro_rules! ceil_index {
    ($index:expr, $size:expr) => {
        ($index + $size - 1) / $size
    };
}

#[macro_export]
macro_rules! ceil {
    ($index:expr, $bound:expr) => {
        (($index + $bound - 1) / $bound) * $bound
    };
}
