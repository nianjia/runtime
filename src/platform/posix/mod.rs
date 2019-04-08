use libc;

pub const PAGE_SHIFT: u64 = 12;
pub const PAGE_SIZE: u64 = 1 << PAGE_SHIFT;

pub fn alloc_pages(num_pages: u32) -> Result<u64, String> {
    let num_bytes = (num_pages as usize) << PAGE_SHIFT;
    let res = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            num_bytes,
            libc::PROT_NONE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    match res {
        libc::MAP_FAILED => Err(format!(
            "mmap(0, {}, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) failed!",
            num_bytes
        )),
        _ => Ok(res as u64),
    }
}

pub fn commit_pages(start_addr: u64, num_pages: u32) -> Result<(), String> {
    assert!(
        start_addr & (PAGE_SIZE - 1) == 0,
        format!("Commit pages at {} is not aligned!", start_addr)
    );
    let num_bytes = (num_pages as usize) << PAGE_SHIFT;
    match unsafe {
        libc::mprotect(
            start_addr as *mut _,
            num_bytes,
            libc::PROT_READ | libc::PROT_WRITE,
        )
    } {
        0 => Ok(()),
        _ => Err(format!(
            "mprotect({}, {},　{}) failed!",
            start_addr,
            num_bytes,
            libc::PROT_READ | libc::PROT_WRITE
        )),
    }
}

pub fn free_pages(start_addr: u64, num_pages: u32) -> Result<(), String> {
    let num_bytes = (num_pages as usize) << PAGE_SHIFT;
    match unsafe { libc::munmap(start_addr as *mut _, num_bytes) } {
        0 => Ok(()),
        _ => Err(format!("munmap({:x}, {:x}) failed!", start_addr, num_bytes)),
    }
}


pub fn copy_memory(dest_addr: u64, value: &[u8]) {
    unsafe { libc::memmove(dest_addr as *mut _, value.as_ptr() as *const _, value.len())};
}