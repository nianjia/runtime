use crate::platform;
use crate::runtime::compartment::Compartment;
use crate::wasm::types::MemoryType;
use crate::wasm::PAGE_SHIFT as WASM_PAGE_SHIFT;
use crate::wasm::PAGE_SIZE as WASM_PAGE_SIZE;

pub struct Memory {
    start_addr: u64,
    cur_pages: u32,
    min_pages: u32,
    max_pages: u32,
}

impl Drop for Memory {
    fn drop(&mut self) {}
}

//TODO: figure out whether the address space is 32bit or 64bit, which stated in
//      WebAssembly Specification.
impl Memory {
    pub fn grow_pages(&mut self, num_pages: u32) -> Result<u32, String> {
        let prev_pages = self.cur_pages;
        if num_pages > self.max_pages || prev_pages + num_pages > self.max_pages {
            return Err("The number of pages is exceeding address limit".to_string());
        }
        platform::commit_pages(self.start_addr, num_pages)?;
        self.cur_pages += prev_pages + num_pages;
        Ok(prev_pages)
    }

    pub fn drop_all_pages(&mut self) {
        platform::free_pages(self.start_addr, self.max_pages + 1);
    }

    pub fn copy_into_data(&mut self, offset: u64, value: &[u8]) -> Result<(), String> {
        let len = value.len() as u64;
        let max_bytes = (self.max_pages as u64) << WASM_PAGE_SHIFT;
        if offset > max_bytes || len + offset > max_bytes {
            return Err("the data's len is too long.".to_string());
        }
        platform::copy_memory(self.start_addr + offset, value);
        Ok(())
    }
}

pub fn create_memory(compartment: &Compartment, ty: &MemoryType) -> Result<Memory, String> {
    let max_pages = ty.max_pages().unwrap_or(ty.min_pages());
    let start_addr = platform::alloc_pages(max_pages + 1)?;

    // TODO: the `cur_pages` is temporarily smaller than `min_pages`,
    // and causes some inconsistency.
    let mut memory = Memory {
        start_addr,
        cur_pages: 0,
        min_pages: ty.min_pages(),
        max_pages,
    };
    memory.grow_pages(ty.min_pages())?;
    Ok(memory)
}
