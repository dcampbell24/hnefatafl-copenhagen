# Install:
# choco install nsis
# choco install rust

# Run:
# https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe
# Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy Bypass

cargo build --release --example hnefatafl-client --no-default-features
& 'C:\Program Files (x86)\NSIS\Bin\makensis.exe' .\hnefatafl-client-installer.nsi
