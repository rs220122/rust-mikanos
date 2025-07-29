// 一旦ダミーのグローバルアロケータを定義する。
// メモリをアロケートしたい場合は、usb/memoryのLocalBumpAllocatorを使う

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

/// ★ ヒープを一切返さないダミー
struct NullAllocator;

unsafe impl GlobalAlloc for NullAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static GLOBAL: NullAllocator = NullAllocator;

/// 要求サイズが 0 以外なら panic でもループでも OK
#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {}
}
