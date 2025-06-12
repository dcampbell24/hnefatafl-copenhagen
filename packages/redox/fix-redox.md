# Redox Build Fails

## Packages Fail to Build

`redox/config/desktop.toml`

```toml
cosmic-edit = "binary"
#shared-mime-info = {}
```

## linking error with `x86_64-unknown-redox-gcc`

```sh
make pull
touch relibc
rm -rf prefix
make prefix cr.drivers cr.drivers-initfs image
```

## error: rustc interrupted by SIGSEGV

```fish
set --export RUST_MIN_STACK 16777216; make ucr.hnefatafl-copenhagen
```
