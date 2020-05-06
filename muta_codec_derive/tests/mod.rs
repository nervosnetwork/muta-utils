mod mock_types;

use muta_protocol::fixed_codec::FixedCodec;

use crate::mock_types::{Hash, Hex, SignedTransaction, TupleStructWithVec};

macro_rules! test_fixed_codec {
    ($($type:ident),+) => (
        $(
            let data = $type::new();
            assert_eq!(
                data,
                FixedCodec::decode_fixed(data.encode_fixed().unwrap()).unwrap()
            );
        )+
    )
}

#[test]
fn test_fixed_codec() {
    test_fixed_codec!(Hex, Hash, SignedTransaction, TupleStructWithVec);
}
