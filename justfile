set shell := ["bash", "-uc"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# help
help:
  @just --list

# build
build:
  @cargo build --release

# lint
lint:
  @cargo clippy

# run
run:
  @cargo run --release
