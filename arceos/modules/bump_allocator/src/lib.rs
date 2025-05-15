#![no_std]

use allocator::{AllocError, BaseAllocator, ByteAllocator, PageAllocator};
use core::ptr::NonNull;

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
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    byte_end: usize,
    page_end: usize,
    count: usize,
    start: usize,
    size: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            byte_end: 0,
            page_end: 0,
            count: 0,
            start: 0,
            size: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.byte_end = start;
        self.page_end = start + size;
        self.count = 0;
        self.start = start;
        self.size = size;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        unimplemented!()
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let align = layout.align();
        let start = self.byte_end.next_multiple_of(align);
        self.byte_end = start + layout.size();
        if self.byte_end > self.page_end {
            return Err(AllocError::NoMemory);
        }
        unsafe { Ok(NonNull::new_unchecked(start as *mut u8)) }
    }
    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        self.count = self.count - 1;
        if self.count == 0 {
            self.byte_end = self.start;
        }
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

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        if self.count == 0 {
            self.page_end = self.page_end - num_pages * PAGE_SIZE;
            self.count = num_pages;
        }
        if self.count == 0 {
            return Err(AllocError::NoMemory);
        }
        self.count = self.count - 1;
        Ok(self.page_end)
    }
    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        self.count = self.count + 1;
        if self.count == 0 {
            self.page_end = self.page_end + num_pages * PAGE_SIZE;
        }
    }
    fn total_pages(&self) -> usize {
        self.size / PAGE_SIZE
    }
    fn used_pages(&self) -> usize {
        self.total_pages() - self.available_pages()
    }
    fn available_pages(&self) -> usize {
        (self.page_end - self.byte_end) / PAGE_SIZE
    }
}