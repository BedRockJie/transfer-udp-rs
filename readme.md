# Transfer-rs
Rust实现的通信工具和协议解析封装工具


# How to Build
## x86 Linux or Windows
```Bash
cargo build --release

```

## aarch64 Linux
**重要** 交叉编译需要注意工具链路径，在`.cargo/config.toml`中修改
```Bash
cargo build --target aarch64-unknown-linux-gnu --release

```