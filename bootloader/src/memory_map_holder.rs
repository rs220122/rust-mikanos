use crate::uefi::memory::EfiMemoryDescriptor;

pub const MEMORY_MAP_BUFFER_SIZE: usize = 4096 * 4; // 4096バイトのページを4つ分

pub struct MemoryMapHolder {
    pub memory_map_buffer: [u8; MEMORY_MAP_BUFFER_SIZE],
    pub memory_map_size: usize,
    pub map_key: usize,
    // EfiMemoryDescriptorのサイズを取得するのではなく、ここで指定する
    // これは、EfiMemoryDescriptorが今後の拡張性により、サイズが変わる可能性があるためである。
    // ただ、structをそのまま使うと、descriptorの取得する位置がずれてしまうためである。
    pub descriptor_size: usize,
    pub descriptor_version: u32,
}

// メモリマップの一行ずつを取得するイテレータ
pub struct MemoryMapIterator<'a> {
    map: &'a MemoryMapHolder,
    offset: usize, // offset
}

impl<'a> Iterator for MemoryMapIterator<'a> {
    type Item = &'a EfiMemoryDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.map.memory_map_size {
            None
        } else {
            let e: &EfiMemoryDescriptor = unsafe {
                // memory_map_bufferはu8の配列なので、
                // ポインタをEfiMemoryDescriptorにキャストして、offset分だけずらす
                &*(self.map.memory_map_buffer.as_ptr().add(self.offset)
                    as *const EfiMemoryDescriptor)
            };
            self.offset += self.map.descriptor_size; // 次の行へ進む
            Some(e)
        }
    }
}

impl MemoryMapHolder {
    pub const fn new() -> MemoryMapHolder {
        MemoryMapHolder {
            memory_map_buffer: [0; MEMORY_MAP_BUFFER_SIZE],
            memory_map_size: MEMORY_MAP_BUFFER_SIZE,
            map_key: 0,
            descriptor_size: core::mem::size_of::<EfiMemoryDescriptor>(),
            descriptor_version: 0,
        }
    }

    pub fn iter(&self) -> MemoryMapIterator {
        MemoryMapIterator {
            map: self,
            offset: 0,
        }
    }
}
