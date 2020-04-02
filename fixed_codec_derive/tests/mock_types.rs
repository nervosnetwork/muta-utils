#![allow(clippy::new_without_default)]

use fixed_codec_derive::RlpFixedCodec;
use muta_protocol::fixed_codec::{FixedCodec, FixedCodecError};
use muta_protocol::{Bytes, ProtocolResult};
use rand::random;

const HASH_LEN: usize = 32;

type JsonString = String;

#[derive(Clone, Debug, RlpFixedCodec, PartialEq, Eq, Copy)]
pub struct Hash([u8; HASH_LEN]);

impl Hash {
	pub fn new() -> Self {
		let bytes = (0..32).map(|_| random::<u8>()).collect::<Vec<_>>();
		let mut out = [0u8; HASH_LEN];
		out.copy_from_slice(&bytes);
		Hash(out)
	}
}

#[derive(Clone, Debug, RlpFixedCodec, PartialEq, Eq)]
pub struct Hex(String);

impl Hex {
	pub fn new() -> Self {
		let temp = "0x".to_owned() + &String::from("muta-dev");
		Self(temp)
	}
}

#[derive(Clone, Debug, RlpFixedCodec, PartialEq, Eq)]
pub struct SignedTransaction {
	raw: JsonString,
	tx_hash: Hash,
	pubkey: Bytes,
	signature: Bytes,
}

impl SignedTransaction {
	pub fn new() -> Self {
		SignedTransaction {
			raw: JsonString::from("muta-dev"),
			tx_hash: Hash::new(),
			pubkey: random_bytes(32),
			signature: random_bytes(64),
		}
	}
}

fn random_bytes(len: usize) -> Bytes {
	let temp = (0..len).map(|_| random::<u8>()).collect::<Vec<_>>();
	Bytes::from(temp)
}
