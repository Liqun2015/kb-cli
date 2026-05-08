@echo off
chcp 65001 >nul

REM ============================================================
REM build_release.bat
REM
REM Deterministic Rust build helper for Windows.
REM Run from the project root or call as scripts\build_release.bat.
REM ============================================================

echo.
echo ===== Checking Rust formatting =====
cargo fmt --check
if errorlevel 1 (
    echo.
    echo cargo fmt --check failed.
    exit /b 1
)

echo.
echo ===== Running tests =====
cargo test
if errorlevel 1 (
    echo.
    echo cargo test failed.
    exit /b 1
)

echo.
echo ===== Checking project =====
cargo check
if errorlevel 1 (
    echo.
    echo cargo check failed.
    exit /b 1
)

echo.
echo ===== Building release executable =====
cargo build --release
if errorlevel 1 (
    echo.
    echo cargo build --release failed.
    exit /b 1
)

echo.
echo Build succeeded.
echo Executable:
echo target\release\kb.exe
