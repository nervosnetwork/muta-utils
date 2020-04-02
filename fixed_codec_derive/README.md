# Muta Fixed Codec Derive

## FixedCodec Trait

```rust
pub trait FixedCodec: Sized {
    fn encode_fixed(&self) -> ProtocolResult<Bytes>;

    fn decode_fixed(bytes: Bytes) -> ProtocolResult<Self>;
}
```

## #[derive(RlpFixedCodec)]

Use rlp to derive `FixedCodec` trait.
