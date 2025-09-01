# 精简的开发脚本

param(
    [Parameter(Position=0)]
    [string]$Command = "help"
)

function Write-Step { param([string]$Message); Write-Host "==> $Message" -ForegroundColor Blue }
function Write-Success { param([string]$Message); Write-Host "[OK] $Message" -ForegroundColor Green }
function Write-Error { param([string]$Message); Write-Host "[ERROR] $Message" -ForegroundColor Red }

# 质量检查（集成所有检查）
function Invoke-Check {
    Write-Step "Running all quality checks..."
    
    # 格式检查
    Write-Step "Checking format..."
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { Write-Error "Format check failed"; exit 1 }
    
    # Clippy检查
    Write-Step "Running Clippy..."
    cargo clippy --workspace --all-targets -- -D clippy::correctness -D clippy::suspicious -W clippy::complexity
    if ($LASTEXITCODE -ne 0) { Write-Error "Clippy failed"; exit 1 }
    
    # 编译检查
    Write-Step "Checking compilation..."
    cargo check --workspace
    if ($LASTEXITCODE -ne 0) { Write-Error "Compilation check failed"; exit 1 }
    
    Write-Success "All quality checks passed!"
}

# 格式化代码
function Invoke-Format {
    Write-Step "Formatting code..."
    cargo fmt --all
    Write-Success "Code formatted"
}

# 运行测试
function Invoke-Test {
    Write-Step "Running tests..."
    cargo test --workspace
    if ($LASTEXITCODE -ne 0) { Write-Error "Tests failed"; exit 1 }
    Write-Success "All tests passed"
}

# 构建桌面应用
function Invoke-BuildDesktop {
    Write-Step "Building desktop app..."
    
    # 检查并安装工具
    if (-not (Get-Command "trunk" -ErrorAction SilentlyContinue)) {
        Write-Step "Installing Trunk..."
        cargo install trunk --locked
    }
    if (-not (Get-Command "cargo-tauri" -ErrorAction SilentlyContinue)) {
        Write-Step "Installing Tauri CLI..."
        cargo install tauri-cli --locked
    }
    
    # 构建
    Write-Step "Building frontend..."
    trunk build --release
    if ($LASTEXITCODE -ne 0) { exit 1 }
    
    Write-Step "Building Tauri app..."
    cargo tauri build
    if ($LASTEXITCODE -ne 0) { exit 1 }
    
    Write-Success "Desktop app built successfully"
}

# 构建Android应用
function Invoke-BuildAndroid {
    Write-Step "Building Android app..."
    
    # 检查Android环境
    if (-not $env:ANDROID_HOME) {
        Write-Error "ANDROID_HOME not set. Please install Android SDK."
        exit 1
    }
    
    # 安装Android targets
    Write-Step "Adding Android targets..."
    rustup target add aarch64-linux-android armv7-linux-androideabi
    
    # 构建
    Invoke-BuildDesktop  # 先构建前端
    Write-Step "Building Android APK..."
    cargo tauri android build --apk
    if ($LASTEXITCODE -ne 0) { exit 1 }
    
    Write-Success "Android app built successfully"
}

# 清理
function Invoke-Clean {
    Write-Step "Cleaning build artifacts..."
    cargo clean
    if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" }
    Write-Success "Cleaned"
}

# 帮助
function Show-Help {
    Write-Host "Available commands:" -ForegroundColor Cyan
    Write-Host "  check       - Run all quality checks (fmt + clippy + compile)"
    Write-Host "  fmt         - Format code"
    Write-Host "  test        - Run tests"
    Write-Host "  build       - Build desktop app"
    Write-Host "  android     - Build Android app"
    Write-Host "  clean       - Clean build artifacts"
    Write-Host "  help        - Show this help"
}

# 主逻辑
switch ($Command.ToLower()) {
    "check" { Invoke-Check }
    "fmt" { Invoke-Format }
    "test" { Invoke-Test }
    "build" { Invoke-BuildDesktop }
    "android" { Invoke-BuildAndroid }
    "clean" { Invoke-Clean }
    default { Show-Help }
}
