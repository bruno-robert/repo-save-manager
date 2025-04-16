# Cross-Compiling from M1 MacOS

## For linux

[Source Guide](https://medium.com/better-programming/cross-compiling-rust-from-mac-to-linux-7fad5a454ab1)

```sh
rustup target add x86_64-unknown-linux-gnu
brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu
```

Add to `Users/bruno/.local/share/cargo/config.toml`

```toml
[target.x86_64-unknown-linux-gnu]
linker = "x86_64-unknown-linux-gnu-gcc"
```

Build
`cargo build --release --target x86_64-unknown-linux-gnu`

## For Windows

Install [cargo-xwin](https://github.com/rust-cross/cargo-xwin) and use it.

```sh
cargo install --locked cargo-xwin
rustup target add x86_64-pc-windows-msvc
cargo xwin build --release --target x86_64-pc-windows-msvc
```
