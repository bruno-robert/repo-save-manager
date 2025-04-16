# REPO Save Manager

Repo save manager (RSM) is a GUI application, built in rust.
It allows you to view, backup and restore your saves.

![Screenshot of RSM](./docs/screenshot_1.png)

Currently supported platforms are

- Windows (x86)
- Linux (x86)
- MacOS (apple silicon)

## Cross-Compiling from M1 MacOS

### For linux

[Source Guide](https://medium.com/better-programming/cross-compiling-rust-from-mac-to-linux-7fad5a454ab1)

```sh
rustup target add x86_64-unknown-linux-gnu
brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu
```

Add to `~/.cargo/config.toml`

```toml
[target.x86_64-unknown-linux-gnu]
linker = "x86_64-unknown-linux-gnu-gcc"
```

Build
`cargo build --release --target x86_64-unknown-linux-gnu`

### For Windows

Install [cargo-xwin](https://github.com/rust-cross/cargo-xwin) and use it.

```sh
cargo install --locked cargo-xwin
rustup target add x86_64-pc-windows-msvc
cargo xwin build --release --target x86_64-pc-windows-msvc
```
