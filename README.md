# Envoy

Envoy is collection of examples, snippets and tools targeted for the Embassy Rust Project.

## Broadcast

`envoy::broadcast` is a alternative to `embassy_sync::PubSub`, where each subscribers has its own queue instead of a shared queue for the enitire channel.


## Enum Channel

`envoy::enum_channel` similar to `envoy::broadcast`. However this messaging system uses enums to allow a single receiver to `.await` on messages from multiple different channels that produces types present in the receivers's enum.  
