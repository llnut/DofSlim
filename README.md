# DofSlim

An LD_PRELOAD library that replaces the hardcoded 1000-slot `TCPUser` pool in `df_channel_r` and `df_bridge_r` with a runtime-configurable size. A pool of 3 saves about 2.5 GB of physical memory per service.

## Building

```bash
rustup target add i686-unknown-linux-gnu
sudo yum install glibc-devel.i686      # or distro equivalent
cargo build --release --target i686-unknown-linux-gnu
```

Output: `target/i686-unknown-linux-gnu/release/libdofslim.so`.

## Usage

```bash
export CLIENT_POOL_SIZE=64
LD_PRELOAD=./libdofslim.so ./df_bridge_r
```

Expect one line on stderr per patched target:

```text
[dofslim] df_bridge_r: 6/6 patches applied, pool_size=64
```

`CLIENT_POOL_SIZE` accepts integers in `[3, 1000]`; anything else leaves the binary as-is. Typical values: 3 for single-user, 10–20 for small servers, 256 for medium deployments, 1000 to disable the patch.

## License

MIT License - see the [LICENSE](LICENSE) file for details.
