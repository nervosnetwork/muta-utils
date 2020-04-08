use fixed_codec_derive::RlpFixedCodec;
use muta_protocol::fixed_codec::{FixedCodec, FixedCodecError};
use muta_protocol::{Bytes, ProtocolResult};

#[derive(Clone, Debug, RlpFixedCodec)]
pub struct Foo {
	string: String,
	bytes: Bytes,
	list: Vec<Bytes>,
}

impl Foo {
	fn new() -> Foo {
		Foo {
			string: String::from("muta-dev"),
			bytes: Bytes::from(vec![0, 1, 2]),
			list: vec![Bytes::default()],
		}
	}
}

fn main() {
	let bytes = Foo::new().encode_fixed().unwrap();
	println!("{:?}", bytes);
}
