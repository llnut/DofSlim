use ctor::ctor;
use libc::{_SC_PAGESIZE, PROT_EXEC, PROT_READ, PROT_WRITE, c_void, memcpy, mprotect, sysconf};
use std::env;

fn get_client_num() -> Option<u32> {
    env::var("DF_CLIENT_NUM")
        .ok()?
        .parse()
        .ok()
        .filter(|&n| n > 2)
}

unsafe fn safe_write<T>(addr: usize, data: &T) -> bool {
    let size = std::mem::size_of::<T>();
    let pagesize = unsafe { sysconf(_SC_PAGESIZE) as usize };
    let page_start = addr & !(pagesize - 1);
    let page_ptr = page_start as *mut c_void;
    unsafe {
        if mprotect(page_ptr, pagesize, PROT_READ | PROT_WRITE | PROT_EXEC) != 0 {
            eprintln!("hook failed: mprotect failed at page {:x}", page_start);
            return false;
        }
        memcpy(addr as *mut c_void, data as *const T as *const c_void, size);
    }
    true
}

#[cfg(feature = "channel")]
#[ctor]
fn hook_channel() {
    let client_num = get_client_num().unwrap_or(1000);
    if client_num == 1000 {
        return;
    }
    let val: u32 = 4 + 0x140060u32 * client_num;
    let for_num = client_num - 1;

    unsafe {
        safe_write(0x0805380D, &val); // 4 bytes
        safe_write(0x0805381C, &client_num); // 4 bytes
        safe_write(0x08053829, &for_num); // 4 bytes
        safe_write(0x080538A4, &client_num); // 4 bytes
        safe_write(0x08053964, &for_num); // 4 bytes
    }
    eprintln!("[df_channel_hook] Patched client limit to {}", client_num);
}

#[cfg(feature = "bridge")]
#[ctor]
fn hook_bridge() {
    let client_num = get_client_num().unwrap_or(1000);
    if client_num == 1000 {
        return;
    }
    let val: u32 = 4 + 0x140060u32 * client_num;
    let for_num = client_num - 1;

    unsafe {
        safe_write(0x08058207, &val); // 4 bytes
        safe_write(0x08058216, &client_num); // 4 bytes
        safe_write(0x08058223, &for_num); // 4 bytes
        safe_write(0x0805829E, &client_num); // 4 bytes
        safe_write(0x0805835E, &for_num); // 4 bytes
    }
    eprintln!("[df_bridge_hook] Patched client limit to {}", client_num);
}
