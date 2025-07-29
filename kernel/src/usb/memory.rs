use core::alloc::{AllocError, Allocator, Layout};
use core::cell::UnsafeCell;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};

/// no_std環境用のシンプルなBump Allocator
/// 固定サイズのメモリプールから順番にメモリを割り当てる
pub struct BumpAllocator<const SIZE: usize> {
    memory: UnsafeCell<[u8; SIZE]>,
    offset: AtomicUsize,
}

impl<const SIZE: usize> BumpAllocator<SIZE> {
    /// 新しいBump Allocatorを作成
    pub const fn new() -> Self {
        Self {
            memory: UnsafeCell::new([0; SIZE]),
            offset: AtomicUsize::new(0),
        }
    }

    /// メモリを割り当てる
    pub fn alloc(&self, layout: Layout) -> Option<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();

        // 現在のオフセットを取得
        let mut current_offset = self.offset.load(Ordering::Acquire);

        loop {
            // アライメントを考慮してオフセットを調整
            let aligned_offset = align_up(current_offset, align);
            let new_offset = aligned_offset + size;

            // メモリプールのサイズを超えないかチェック
            if new_offset > SIZE {
                return None; // メモリ不足
            }

            // アトミックに更新を試行
            match self.offset.compare_exchange_weak(
                current_offset,
                new_offset,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // 成功：メモリポインタを返す
                    unsafe {
                        let memory = &*self.memory.get();
                        let ptr = memory.as_ptr().add(aligned_offset);
                        return Some(NonNull::new_unchecked(ptr as *mut u8));
                    }
                }
                Err(offset) => {
                    // 失敗：他のスレッドが先に更新した
                    current_offset = offset;
                    continue;
                }
            }
        }
    }
}

unsafe impl<const SIZE: usize> Allocator for BumpAllocator<SIZE> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match self.alloc(layout) {
            Some(ptr) => {
                // NonNull<[u8]>を作成（ポインタとサイズを組み合わせ）
                let slice_ptr = core::ptr::slice_from_raw_parts_mut(ptr.as_ptr(), layout.size());
                Ok(NonNull::new(slice_ptr).unwrap())
            }
            None => Err(AllocError),
        }
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // Bump allocatorは個別の解放をサポートしない
        // リセットすることで全てのメモリを一度に解放する
    }
}

unsafe impl<const SIZE: usize> Send for BumpAllocator<SIZE> {}
unsafe impl<const SIZE: usize> Sync for BumpAllocator<SIZE> {}

/// アライメントに合わせて値を切り上げる
const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

/// 便利なtype alias
pub type BumpAllocator1K = BumpAllocator<1024>;
pub type BumpAllocator4K = BumpAllocator<4096>;
pub type BumpAllocator64K = BumpAllocator<65536>;
