use std::alloc::Layout;
use std::mem::size_of;

pub unsafe fn alloc(size: usize) -> *mut u8 {
    let size = size + size_of::<usize>();
    let layout = Layout::from_size_align_unchecked(size, size_of::<usize>());
    let ptr = std::alloc::alloc(layout);
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    *(ptr as *mut usize) = size;
    ptr.offset(size_of::<usize>() as isize)
}

pub unsafe fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    if ptr.is_null() {
        alloc(size)
    } else {
        let size = size + size_of::<usize>();
        let ptr = ptr.offset(-(size_of::<usize>() as isize));
        let old_size = *(ptr as *mut usize);
        let layout = Layout::from_size_align_unchecked(old_size, size_of::<usize>());
        let ptr = std::alloc::realloc(ptr, layout, size);
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        *(ptr as *mut usize) = size;
        ptr.offset(size_of::<usize>() as isize)
    }
}

pub unsafe fn free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    let ptr = ptr.offset(-(size_of::<usize>() as isize));
    let size = *(ptr as *mut usize);
    let layout = Layout::from_size_align_unchecked(size, size_of::<usize>());
    std::alloc::dealloc(ptr, layout);
}