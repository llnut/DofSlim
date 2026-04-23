//! Patch sites for each supported binary.

/// Original compile-time pool capacity in both binaries.
pub const ORIGINAL_POOL_SIZE: u32 = 1000;

/// `sizeof(TCPUser)`, as used by the allocation-size expression.
const TCPUSER_SIZE: u32 = 0x14_0060;

#[derive(Clone, Copy)]
pub enum PatchKind {
    /// Slot count. Original 1000.
    PoolSize,
    /// Zero-based loop bound `pool_size - 1`. Original 999.
    LoopBound,
    /// `operator new[]` argument `4 + TCPUSER_SIZE * pool_size`. Original 0x4E217704.
    AllocSize,
}

impl PatchKind {
    /// Value this site should hold for a given pool size.
    pub const fn compute(self, pool_size: u32) -> u32 {
        match self {
            PatchKind::PoolSize => pool_size,
            PatchKind::LoopBound => pool_size.saturating_sub(1),
            PatchKind::AllocSize => 4 + TCPUSER_SIZE * pool_size,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Patch {
    /// Address of the imm32 operand.
    pub addr: usize,
    pub kind: PatchKind,
}

pub struct Target {
    /// Basename of `/proc/self/exe` matched at startup.
    pub name: &'static str,
    pub patches: &'static [Patch],
}

/// `df_channel_r`. Sites in source order: alloc, header count, ctor loop,
/// exception unwind, printf arg, free-queue push loop.
const DF_CHANNEL_R: &[Patch] = &[
    Patch {
        addr: 0x0805_380D,
        kind: PatchKind::AllocSize,
    },
    Patch {
        addr: 0x0805_381C,
        kind: PatchKind::PoolSize,
    },
    Patch {
        addr: 0x0805_3829,
        kind: PatchKind::LoopBound,
    },
    Patch {
        addr: 0x0805_385B,
        kind: PatchKind::LoopBound,
    },
    Patch {
        addr: 0x0805_38A4,
        kind: PatchKind::PoolSize,
    },
    Patch {
        addr: 0x0805_3964,
        kind: PatchKind::LoopBound,
    },
];

/// `df_bridge_r`. Same sites, different base.
const DF_BRIDGE_R: &[Patch] = &[
    Patch {
        addr: 0x0805_8207,
        kind: PatchKind::AllocSize,
    },
    Patch {
        addr: 0x0805_8216,
        kind: PatchKind::PoolSize,
    },
    Patch {
        addr: 0x0805_8223,
        kind: PatchKind::LoopBound,
    },
    Patch {
        addr: 0x0805_8255,
        kind: PatchKind::LoopBound,
    },
    Patch {
        addr: 0x0805_829E,
        kind: PatchKind::PoolSize,
    },
    Patch {
        addr: 0x0805_835E,
        kind: PatchKind::LoopBound,
    },
];

pub const TARGETS: &[Target] = &[
    Target {
        name: "df_channel_r",
        patches: DF_CHANNEL_R,
    },
    Target {
        name: "df_bridge_r",
        patches: DF_BRIDGE_R,
    },
];
