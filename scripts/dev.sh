#!/usr/bin/env bash
# 精简的开发脚本

set -e

RED='\033[0;31m'; GREEN='\033[0;32m'; BLUE='\033[0;34m'; NC='\033[0m'

print_step() { echo -e "${BLUE}==>${NC} $1"; }
print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }

# 质量检查（集成所有检查）
check() {
    print_step "Running all quality checks..."
    
    print_step "Checking format..."
    cargo fmt --all -- --check || { print_error "Format check failed"; exit 1; }
    
    print_step "Running Clippy..."
    cargo clippy --workspace --all-targets -- -D clippy::correctness -D clippy::suspicious -W clippy::complexity || { print_error "Clippy failed"; exit 1; }
    
    print_step "Checking compilation..."
    cargo check --workspace || { print_error "Compilation check failed"; exit 1; }
    
    print_success "All quality checks passed!"
}

# 格式化代码
fmt() {
    print_step "Formatting code..."
    cargo fmt --all
    print_success "Code formatted"
}

# 运行测试
test() {
    print_step "Running tests..."
    cargo test --workspace || { print_error "Tests failed"; exit 1; }
    print_success "All tests passed"
}

# 构建桌面应用
build() {
    print_step "Building desktop app..."
    
    # 检查并安装工具
    if ! command -v trunk &> /dev/null; then
        print_step "Installing Trunk..."
        cargo install trunk --locked
    fi
    if ! command -v cargo-tauri &> /dev/null; then
        print_step "Installing Tauri CLI..."
        cargo install tauri-cli --locked
    fi
    
    # 构建
    print_step "Building frontend..."
    trunk build --release
    
    print_step "Building Tauri app..."
    cargo tauri build
    
    print_success "Desktop app built successfully"
}

# 构建Android应用
android() {
    print_step "Building Android app..."
    
    # 检查Android环境
    if [ -z "$ANDROID_HOME" ]; then
        print_error "ANDROID_HOME not set. Please install Android SDK."
        exit 1
    fi
    
    # 安装Android targets
    print_step "Adding Android targets..."
    rustup target add aarch64-linux-android armv7-linux-androideabi
    
    # 构建
    build  # 先构建前端
    print_step "Building Android APK..."
    cargo tauri android build --apk
    
    print_success "Android app built successfully"
}

# 清理
clean() {
    print_step "Cleaning build artifacts..."
    cargo clean
    rm -rf dist/
    print_success "Cleaned"
}

# 帮助
help() {
    echo "Available commands:"
    echo "  check       - Run all quality checks (fmt + clippy + compile)"
    echo "  fmt         - Format code"
    echo "  test        - Run tests"
    echo "  build       - Build desktop app"
    echo "  android     - Build Android app"
    echo "  clean       - Clean build artifacts"
    echo "  help        - Show this help"
}

# 主逻辑
case "${1:-help}" in
    check) check ;;
    fmt) fmt ;;
    test) test ;;
    build) build ;;
    android) android ;;
    clean) clean ;;
    *) help ;;
esac
