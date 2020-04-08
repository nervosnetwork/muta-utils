use fixed_codec_derive::RlpFixedCodec;
use muta_protocol::fixed_codec::{FixedCodec, FixedCodecError};
use muta_protocol::{Bytes, ProtocolResult};

#[derive(Clone, Debug, RlpFixedCodec)]
pub struct Foo {
	string: String,
	bytes: Bytes,
	list: Vec<Bytes>,
}

fn main() {}
