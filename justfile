example: 
    HTF_LOG=debug cargo run --package htf --example simple

log:
    tail -f htf.log

init:
    cargo install cargo-expand
