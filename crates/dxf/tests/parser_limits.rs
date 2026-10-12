//! Deterministic hostile-input and size-limit contracts for the standalone DXF parser.
//! All fixtures are synthetic and derived from public DXF group-code rules.

use cadcraft_dxf::{DxfError, Tag, Value, parse, parse_binary, parse_with_limit, write_ascii};

const BINARY_SENTINEL: &[u8] = b"AutoCAD Binary DXF\r\n\x1a\0";

fn binary_fixture() -> Vec<u8> {
    let mut bytes = BINARY_SENTINEL.to_vec();
    bytes.extend_from_slice(&0i16.to_le_bytes());
    bytes.extend_from_slice(b"LINE\0");
    bytes.extend_from_slice(&10i16.to_le_bytes());
    bytes.extend_from_slice(&1.25f64.to_le_bytes());
    bytes.extend_from_slice(&0i16.to_le_bytes());
    bytes.extend_from_slice(b"EOF\0");
    bytes
}

#[test]
fn ascii_limit_is_enforced_without_losing_valid_drawings() {
    let tags = [Tag::s(0, "SECTION"), Tag::s(2, "ENTITIES"), Tag::s(0, "ENDSEC"), Tag::s(0, "EOF")];
    let text = write_ascii(&tags);

    assert!(matches!(parse_with_limit(text.as_bytes(), 3), Err(DxfError::TooLarge { limit: 3, .. })));
    assert_eq!(parse_with_limit(text.as_bytes(), tags.len()).unwrap(), tags);
    assert_eq!(parse(text.as_bytes()).unwrap(), tags);
}

#[test]
fn binary_limit_counts_decoded_groups_not_input_bytes() {
    let bytes = binary_fixture();
    let expected = [Tag::s(0, "LINE"), Tag::f(10, 1.25), Tag::s(0, "EOF")];

    assert!(matches!(parse_with_limit(&bytes, 2), Err(DxfError::TooLarge { limit: 2, .. })));
    assert_eq!(parse_with_limit(&bytes, expected.len()).unwrap(), expected);
    assert_eq!(parse(&bytes).unwrap(), expected);
}

#[test]
fn every_truncated_binary_prefix_stays_bounded_and_never_panics() {
    let bytes = binary_fixture();
    for end in 0..bytes.len() {
        let parsed = std::panic::catch_unwind(|| parse_with_limit(&bytes[..end], 3));
        let result = parsed.unwrap_or_else(|_| panic!("parser panicked after byte {end}"));
        if let Ok(tags) = result {
            assert!(tags.len() <= 3, "byte {end}: parser exceeded the tag limit");
        }
    }
    assert_eq!(parse(&bytes).unwrap().len(), 3);
}

#[test]
fn extended_binary_hex_codes_preserve_bytes() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&255i16.to_le_bytes());
    bytes.extend_from_slice(&1004i16.to_le_bytes());
    bytes.extend_from_slice(&2u8.to_le_bytes());
    bytes.extend_from_slice(&[0xa5, 0xff]);
    bytes.extend_from_slice(&0i16.to_le_bytes());
    bytes.extend_from_slice(b"EOF\0");

    let parsed = parse_binary(&bytes).unwrap();
    assert_eq!(parsed[0], Tag::new(1004, Value::Hex("A5FF".into())));
    assert_eq!(parsed[1], Tag::s(0, "EOF"));
}

#[test]
fn truncated_binary_hex_payload_is_a_structured_error() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&310i16.to_le_bytes());
    bytes.push(3);
    bytes.extend_from_slice(&[0xaa, 0xbb]);

    assert!(matches!(parse_binary(&bytes), Err(DxfError::Eof)));
}

#[test]
fn invalid_ascii_group_code_reports_its_line() {
    let result = parse(b"invalid\nvalue\n0\nEOF\n");
    assert!(matches!(result, Err(DxfError::BadCode { line: 1, .. })));
}


/// A fixed-seed stream of finite typed values is an independent serialization contract: DXF
/// ASCII must preserve each tag's code and typed value through write and read.
#[test]
fn fixed_seed_ascii_tag_roundtrips_are_byte_reproducible() {
    fn next(state: &mut u64) -> u64 {
        *state ^= *state >> 12;
        *state ^= *state << 25;
        *state ^= *state >> 27;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    for seed in [1_u64, 0x9E37_79B9_7F4A_7C15, 0xFEED_BEEF_CAFE_1234] {
        let mut rng = seed;
        let mut tags = Vec::new();
        for index in 0..256 {
            let n = next(&mut rng);
            let tag = match index % 7 {
                0 => Tag::f(10, (n % 100_000) as f64 / 8.0 - 5_000.0),
                1 => Tag::i(62, (n % 256) as i64),
                2 => Tag::i(90, (n % 100_000) as i64),
                3 => Tag::i(160, (n % 1_000_000) as i64),
                4 => Tag::new(290, Value::Bool(n & 1 == 1)),
                5 => Tag::new(310, Value::Hex(format!("{:04X}", n as u16))),
                _ => Tag::s(1, if n & 1 == 0 { "ASCII" } else { "Café ▵ 😀" }),
            };
            tags.push(tag);
        }
        tags.push(Tag::s(0, "EOF"));
        let encoded = write_ascii(&tags);
        assert!(encoded.is_ascii(), "seed {seed}: DXF ASCII output contains non-ASCII bytes");
        assert_eq!(write_ascii(&tags), encoded, "seed {seed}: writer is nondeterministic");
        assert_eq!(parse(encoded.as_bytes()).unwrap(), tags, "seed {seed}: DXF roundtrip changed the tags");
    }
}

/// Mutate every input byte of a known-small binary DXF without external fuzz services.
/// Valid fragments may parse successfully; they must never exceed the fixed tag limit,
/// crash, or return different results for identical bytes.
#[test]
fn binary_bit_flip_replay_is_deterministic_and_memory_bounded() {
    let original = binary_fixture();
    for byte_index in 0..original.len() {
        let mut bytes = original.clone();
        bytes[byte_index] ^= 0x80;
        let one = std::panic::catch_unwind(|| parse_with_limit(&bytes, 3))
            .unwrap_or_else(|_| panic!("DXF parser panicked after bit flip at byte {byte_index}"));
        let two = parse_with_limit(&bytes, 3);
        assert_eq!(format!("{one:?}"), format!("{two:?}"), "byte {byte_index}: inconsistent parser result");
        if let Ok(tags) = one {
            assert!(tags.len() <= 3, "byte {byte_index}: parser ignored its limit");
        }
    }
}
