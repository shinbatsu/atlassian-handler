set -e

APP_NAME="handler"
VERSION="1.0.0"

echo "=== Building $APP_NAME v$VERSION ==="

case "${1:-linux}" in
    linux)
        echo "Building for Linux (x86_64)..."
        cargo build --release --target x86_64-unknown-linux-gnu
        cp target/x86_64-unknown-linux-gnu/release/$APP_NAME "$APP_NAME-linux-x86_64"
        ;;
    windows)
        echo "Building for Windows (x86_64)..."
        cargo build --release --target x86_64-pc-windows-gnu
        cp target/x86_64-pc-windows-gnu/release/$APP_NAME.exe "$APP_NAME-windows-x86_64.exe"
        ;;
    macos)
        echo "Building for macOS (x86_64)..."
        cargo build --release --target x86_64-apple-darwin
        cp target/x86_64-apple-darwin/release/$APP_NAME "$APP_NAME-macos-x86_64"
        ;;
    all)
        echo "Building for all platforms..."
        cargo build --release --target x86_64-unknown-linux-gnu
        cargo build --release --target x86_64-pc-windows-gnu
        cargo build --release --target x86_64-apple-darwin
        cp target/x86_64-unknown-linux-gnu/release/$APP_NAME "$APP_NAME-linux-x86_64"
        cp target/x86_64-pc-windows-gnu/release/$APP_NAME.exe "$APP_NAME-windows-x86_64.exe"
        cp target/x86_64-apple-darwin/release/$APP_NAME "$APP_NAME-macos-x86_64"
        ;;
    *)
        echo "Unknown target: $1"
        echo "Usage: ./build.sh [linux|windows|macos|all]"
        exit 1
        ;;
esac

echo "=== Build complete ==="
ls -lh "$APP_NAME"* 2>/dev/null || true