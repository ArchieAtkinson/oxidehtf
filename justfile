example name:
    HT2_LOG=debug cargo run --package htf2 --example {{ name }}

log:
    tail -f htf2.log

init:
    cargo install cargo-expand
