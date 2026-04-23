//! LD_PRELOAD library that shrinks the 1000-slot `TCPUser` pool in
//! `df_channel_r` and `df_bridge_r` to a runtime-configurable size.

mod patch;
mod targets;

use std::env;

use ctor::ctor;

use crate::patch::{read_u32, write_u32};
use crate::targets::{ORIGINAL_POOL_SIZE, TARGETS, Target};

const ENV_POOL_SIZE: &str = "CLIENT_POOL_SIZE";
const MIN_POOL_SIZE: u32 = 3;
const LOG_TAG: &str = "[dofslim]";

enum PoolSizeInput {
    Unset,
    Invalid(String),
    Valid(u32),
}

fn read_pool_size() -> PoolSizeInput {
    let raw = match env::var(ENV_POOL_SIZE) {
        Ok(s) => s,
        Err(_) => return PoolSizeInput::Unset,
    };
    match raw.parse::<u32>() {
        Ok(n) if (MIN_POOL_SIZE..=ORIGINAL_POOL_SIZE).contains(&n) => PoolSizeInput::Valid(n),
        _ => PoolSizeInput::Invalid(raw),
    }
}

fn detect_target() -> Option<&'static Target> {
    let exe = env::current_exe().ok()?;
    let name = exe.file_name()?.to_str()?;
    TARGETS.iter().find(|t| t.name == name)
}

/// # Safety
/// Must run before any application thread starts; calling from `#[ctor]`
/// satisfies this.
unsafe fn apply(target: &Target, pool_size: u32) {
    let mut applied = 0usize;
    for patch in target.patches {
        let expected = patch.kind.compute(ORIGINAL_POOL_SIZE);
        let actual = unsafe { read_u32(patch.addr) };
        if actual != expected {
            eprintln!(
                "{LOG_TAG} {} skip {:#010x}: expected {:#x}, found {:#x}",
                target.name, patch.addr, expected, actual
            );
            continue;
        }
        let new_value = patch.kind.compute(pool_size);
        match unsafe { write_u32(patch.addr, new_value) } {
            Ok(()) => applied += 1,
            Err(err) => eprintln!(
                "{LOG_TAG} {} patch {:#010x} failed: {}",
                target.name, patch.addr, err
            ),
        }
    }
    eprintln!(
        "{LOG_TAG} {}: {}/{} patches applied, pool_size={}",
        target.name,
        applied,
        target.patches.len(),
        pool_size,
    );
}

#[ctor]
fn dofslim_init() {
    // Silent when preloaded into an unrelated process.
    let Some(target) = detect_target() else {
        return;
    };
    let pool_size = match read_pool_size() {
        PoolSizeInput::Unset => {
            eprintln!(
                "{LOG_TAG} {}: {} not set, no patch applied",
                target.name, ENV_POOL_SIZE
            );
            return;
        }
        PoolSizeInput::Invalid(raw) => {
            eprintln!(
                "{LOG_TAG} {}: {}={:?} outside [{}, {}], no patch applied",
                target.name, ENV_POOL_SIZE, raw, MIN_POOL_SIZE, ORIGINAL_POOL_SIZE
            );
            return;
        }
        PoolSizeInput::Valid(n) if n == ORIGINAL_POOL_SIZE => return,
        PoolSizeInput::Valid(n) => n,
    };
    unsafe { apply(target, pool_size) };
}
