#![no_std]

use core::ptr::NonNull;

use allocator::{AllocError, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator {
    start: usize,
    size: usize,
    byte_end: usize,
    page_end: usize,
}

impl EarlyAllocator {
    pub const fn new() -> Self {
        Self {
            start: 0,
            size: 0,
            byte_end: 0,
            page_end: 0,
        }
    }
}

impl BaseAllocator for EarlyAllocator {
    fn init(&mut self, start: usize, size: usize) {
        let mut allocator = Self::new();
        allocator.start = start;
        allocator.size = size;
        allocator.byte_end = 0;
        allocator.page_end = start + size;
    }
    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        // if self.start + self.size != start {
        //     return Err(AllocError::InvalidParam);
        // }
        // self.size += size;
        // self.page_end += size;

        // Ok(())
        todo!()
    }
}

impl ByteAllocator for EarlyAllocator {
    fn alloc(&mut self, layout: core::alloc::Layout) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let align = layout.align();
        let start = self.byte_end.next_multiple_of(align);
        self.byte_end = start + layout.size();
        if self.byte_end > self.page_end {
            return Err(AllocError::NoMemory);
        }

        unsafe { Ok(NonNull::new_unchecked(start as *mut u8)) }
    }
    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        todo!();
    }
    fn total_bytes(&self) -> usize {
        self.size
    }
    fn used_bytes(&self) -> usize {
        self.total_bytes() - self.available_bytes()
    }
    fn available_bytes(&self) -> usize {
        self.page_end - self.byte_end
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> allocator::AllocResult<usize> {
        if num_pages > self.available_bytes() {
            return Err(AllocError::NoMemory);
        }
        
    }
    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        todo!();
    }
    fn total_pages(&self) -> usize {
        self.size / PAGE_SIZE;
    }
    fn used_pages(&self) -> usize {
        self.total_pages() - self.available_pages();
    }
    fn available_pages(&self) -> usize {
        (self.page_end - self.byte_end) / PAGE_SIZE;
    }
}
