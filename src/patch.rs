//! `mprotect`-gated read and write helpers for `.text`.

use libc::{_SC_PAGESIZE, PROT_EXEC, PROT_READ, PROT_WRITE, c_void, mprotect, sysconf};
use std::ptr;

const DEFAULT_PAGE_SIZE: usize = 4096;

fn page_size() -> usize {
    let v = unsafe { sysconf(_SC_PAGESIZE) };
    if v > 0 { v as usize } else { DEFAULT_PAGE_SIZE }
}

/// # Safety
/// `addr` must point to 4 readable bytes.
pub unsafe fn read_u32(addr: usize) -> u32 {
    unsafe { ptr::read_unaligned(addr as *const u32) }
}

/// Writes a little-endian `u32`, remapping the spanned pages RWX and then
/// restoring them to RX. Straddling a page boundary widens the range.
///
/// # Safety
/// No other thread may be executing instructions that overlap the bytes
/// being written.
pub unsafe fn write_u32(addr: usize, value: u32) -> Result<(), &'static str> {
    let ps = page_size();
    let start = addr & !(ps - 1);
    let end = (addr + 4 + ps - 1) & !(ps - 1);
    let len = end - start;

    let unlock = PROT_READ | PROT_WRITE | PROT_EXEC;
    if unsafe { mprotect(start as *mut c_void, len, unlock) } != 0 {
        return Err("mprotect rwx failed");
    }

    unsafe { ptr::write_unaligned(addr as *mut u32, value) };

    let relock = PROT_READ | PROT_EXEC;
    if unsafe { mprotect(start as *mut c_void, len, relock) } != 0 {
        return Err("mprotect rx restore failed");
    }
    Ok(())
}
