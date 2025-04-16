# Get the parent folder path of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cargo build --release --target aarch64-apple-darwin \
    && mv $SCRIPT_DIR/target/aarch64-apple-darwin/release/rsm $SCRIPT_DIR/target/aarch64-apple-darwin/release/rsm-macos-arm
cargo build --release --target x86_64-unknown-linux-gnu \
    && mv $SCRIPT_DIR/target/x86_64-unknown-linux-gnu/release/rsm $SCRIPT_DIR/target/x86_64-unknown-linux-gnu/release/rsm-linux-x86_64
cargo xwin build --release --target x86_64-pc-windows-msvc \
    && mv $SCRIPT_DIR/target/x86_64-pc-windows-msvc/release/rsm.exe $SCRIPT_DIR/target/x86_64-pc-windows-msvc/release/rsm-windows-x86_64.exe
