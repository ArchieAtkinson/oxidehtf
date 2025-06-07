example name:
    OXIDE_HTF_LOG=debug cargo run --package oxidehtf --example {{ name }}

log:
    tail -f oxidehtf.log

init:
    cargo install cargo-expand
