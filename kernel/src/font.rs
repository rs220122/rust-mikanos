use core::ptr::null_mut;

unsafe extern "C" {
    static _binary_hankaku_bin_start: u8;
    static _binary_hankaku_bin_end: u8;
}

// フォントのリストの先頭アドレスを返す。一つのフォントは、[u8; 16]となっている。
pub fn get_font(c: u8) -> *const u8 {
    let index: usize = 16 * c as usize;
    let start = unsafe { &_binary_hankaku_bin_start as *const u8 };
    let end = unsafe { &_binary_hankaku_bin_end as *const u8 };

    let size = unsafe { end.offset_from(start) as usize };

    if index >= size {
        return null_mut();
    }

    return (start as usize + index) as *const u8;
}
