example name:
    HTF_LOG=debug cargo run --package htf --example {{ name }}

log:
    tail -f htf.log

init:
    cargo install cargo-expand
