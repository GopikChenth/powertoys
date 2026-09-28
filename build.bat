@echo off
set "PATH=%LOCALAPPDATA%\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin;%PATH%"
set "RUST_FONTCONFIG_DLOPEN=1"

if "%~1"=="" (
    cargo run -p powertoys-daemon -- run
) else if "%~1"=="check" (
    cargo check --workspace
) else if "%~1"=="check-linux" (
    cargo check --workspace --target x86_64-unknown-linux-gnu
) else if "%~1"=="build" (
    cargo build --workspace
) else if "%~1"=="build-linux" (
    cargo build --workspace --target x86_64-unknown-linux-gnu --release
) else if "%~1"=="release" (
    cargo build --workspace --release
) else (
    cargo run -p powertoys-daemon -- %*
)
