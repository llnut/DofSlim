# DofSlim

Dynamically reduce server memory usage by replacing the hardcoded 1000 client limit with a runtime configurable value (e.g., 64 or 128).

Saves over 2.6 GB of memory for df_bridge_r and df_channel_r services.

## 📌 Background

The original df_bridge_r and df_channel_r binaries statically allocate memory for 1000 client objects at startup (each approximately 1.25 MB in size). Even if your actual concurrent connections are far fewer (e.g., dozens or hundreds), the program still reserves 1.3–2.5 GB of virtual memory, leading to significant resource waste.

This project uses LD_PRELOAD hooking to dynamically patch the following hardcoded constants before the program starts:

    Client count limit: 1000 → custom_value
    Loop boundary: 999 → custom_value - 1
    Memory offset calculation: 4 + 0x140060 * 1000 → 4 + 0x140060 * custom_value

## 🚀 Features

    ✅ Runtime-configurable client limit via DF_CLIENT_NUM environment variable (default: 1000)
    ✅ Supports df_bridge_r and df_channel_r
    ✅ Safe patching: Only modifies constants directly related to client count
    ✅ Significant memory reduction:
        DF_CLIENT_NUM=1 → saves ~2.6 GB physical memory
    ✅ Overflow-safe: Uses saturating_sub to prevent crashes when client_num = 0

## 🛠️ Building

* Prerequisites

    Rust toolchain (rustc >= 1.70)
    cargo
    Target system: 32-bit Linux (original binaries are elf32-i386)

* Steps

    ```bash
    # Clone the repository
    git clone https://github.com/llnut/dof-memory-hook.git
    cd dof-memory-hook

    # Build the 32-bit shared library for df_bridge_r
    cargo build --release --target i686-unknown-linux-gnu --features bridge

    # Build the 32-bit shared library for df_channel_r
    cargo build --release --target i686-unknown-linux-gnu --features channel

    # Output file
    ls -lh target/i686-unknown-linux-gnu/release/libdofslim.so
    ```

    💡 On 64-bit systems, install 32-bit toolchain first:

    ```bash
    sudo yum install glibc-devel.i686
    rustup target add i686-unknown-linux-gnu
    ```

## ▶️ Usage
1. Set client limit (optional)

    ```bash
    # Default is 1000; adjust based on your needs
    export DF_CLIENT_NUM=64   # or 128, 256, etc.
    ```

2. Launch services with LD_PRELOAD

    ```bash
    # For df_bridge_r
    LD_PRELOAD=/path/to/target/i686-unknown-linux-gnu/release/libdofslim.so ./df_bridge_r

    # For df_channel_r
    LD_PRELOAD=/path/to/target/i686-unknown-linux-gnu/release/libdofslim.so ./df_channel_r
    ```

3. Verify it works

    Check stderr output:

    ```text
    [df_bridge_hook] Patched client limit to 64
    [df_channel_hook] Patched client limit to 64
    ```

    Monitor memory usage (compare with unpatched version):

    ```bash
    # Check virtual (VSZ) and physical (RSS) memory
    ps -o pid,vsz,rss,cmd -p $(pgrep df_bridge_r)
    ```

## ⚙️ Environment Variables

|Variable|Description|Default|
|---|---|---|
|DF_CLIENT_NUM|Maximum number of concurrent clients|1000|

    📝 Recommended values:

        Single-person server: 1
        Small server: 10 or 20
        Medium server: 256
        Full compatibility: 1000

## 🔒 Safety Notes

1. If DF_CLIENT_NUM is unset or invalid, the program falls back to 1000 and behaves identically to the original.
2. Tested and stable for values from 1 to 1000.

## 📄 License

MIT License
