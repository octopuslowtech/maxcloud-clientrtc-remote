#!/bin/bash

# Script để build dự án Rust cho Linux và Windows
# Dự án: WebRTC client với actix-web, datachannel-rs, signalrs_client

set -e  # Thoát nếu có lỗi

# Kiểm tra và cài đặt các công cụ cần thiết
check_tools() {
    local tools=("cargo" "rustup" "openssl" "pkg-config" "build-essential")
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            echo "Cài đặt $tool..."
            sudo apt-get install -y "$tool"
        fi
    done

    # Cài đặt thư viện OpenSSL phát triển
    sudo apt-get install -y libssl-dev
}

# Định nghĩa các target
TARGETS=(
    "x86_64-unknown-linux-gnu"  # Linux
    "x86_64-pc-windows-gnu"     # Windows
)

# Thư mục đầu ra
OUTPUT_DIR="build"
mkdir -p "$OUTPUT_DIR"

# Đảm bảo chứng chỉ TLS tồn tại
CERT_FILES=("cert.pem" "key.pem")
for CERT in "${CERT_FILES[@]}"; do
    if [ ! -f "$CERT" ]; then
        echo "Tạo chứng chỉ tự ký..."
        openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes -subj "/CN=localhost"
    fi
done

# Cài các target Rust nếu chưa có
echo "Cài đặt các target Rust..."
for TARGET in "${TARGETS[@]}"; do
    if ! rustup target list | grep -q "$TARGET (installed)"; then
        echo "Cài đặt target $TARGET..."
        rustup target add "$TARGET"
    fi
done

# Cài đặt mingw-w64 cho Windows build
if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    echo "Cài đặt mingw-w64..."
    sudo apt-get install -y mingw-w64
fi

# Build cho từng target
for TARGET in "${TARGETS[@]}"; do
    echo "Đang build cho $TARGET..."

    # Thiết lập biến môi trường cho Windows build
    if [[ "$TARGET" == *"windows"* ]]; then
        export PKG_CONFIG_ALLOW_CROSS=1
        export PKG_CONFIG_PATH=/usr/x86_64-w64-mingw32/lib/pkgconfig
    fi

    # Build release
    cargo build --release --target "$TARGET"

    # Xác định tên binary và thư mục đích
    if [[ "$TARGET" == *"windows"* ]]; then
        BINARY="maxcloud-clientrtc-remote.exe"
    else
        BINARY="maxcloud-clientrtc-remote"
    fi
    TARGET_DIR="target/$TARGET/release"
    DEST_DIR="$OUTPUT_DIR/$TARGET"

    # Tạo thư mục đích
    mkdir -p "$DEST_DIR"

    # Copy binary
    cp "$TARGET_DIR/$BINARY" "$DEST_DIR/"
    echo "Đã copy $BINARY sang $DEST_DIR"

    # Copy chứng chỉ TLS
    cp cert.pem key.pem "$DEST_DIR/"
    echo "Đã copy cert.pem và key.pem sang $DEST_DIR"
done

# Kiểm tra kết quả
echo "Build hoàn tất! Kiểm tra thư mục $OUTPUT_DIR:"
ls -R "$OUTPUT_DIR"

# Hướng dẫn chạy
echo ""
echo "Để chạy trên mỗi nền tảng:"
echo "- Linux: cd $OUTPUT_DIR/x86_64-unknown-linux-gnu && ./maxcloud-clientrtc-remote"
echo "- Windows: cd $OUTPUT_DIR/x86_64-pc-windows-gnu && maxcloud-clientrtc-remote.exe"
echo ""
echo "Kiểm tra endpoint:"
echo "- curl https://localhost:8080/hello --cacert cert.pem"
echo "- curl \"https://localhost:8080/login?key=abc123\" --cacert cert.pem"

# Thêm quyền thực thi cho script
chmod +x "$0"
