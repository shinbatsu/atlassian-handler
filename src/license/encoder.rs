use base64::{Engine as _, engine::general_purpose};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use num_bigint::{BigInt, BigUint, Sign, RandBigInt};
use rand::thread_rng;
use sha1::{Sha1, Digest};
use std::io::Write;

pub struct DsaPrivateKey {
    p: BigUint,
    q: BigUint,
    g: BigUint,
    x: BigUint,
}

impl DsaPrivateKey {
    fn new() -> Self {
        let p_hex = "fd7f53811d75122952df4a9c2eece4e7f611b7523cef4400c31e3f80b6512669455d402251fb593d8d58fabfc5f5ba30f6cb9b556cd7813b801d346ff26660bb6b9950a5a49f9fe8047b1022c24fbba9d7feb7c61bf83b57e7c6a8a6150f04fb83f6d3c51ec3023554135a169132f675f3ae2b61d72aeff22203199dd14801c7";
        let p = BigUint::parse_bytes(p_hex.as_bytes(), 16).expect("Invalid P hex");

        let q_hex = "009760508f15230bccb292b982a2ee102fc16073d4";
        let q = BigUint::parse_bytes(q_hex.as_bytes(), 16).expect("Invalid Q hex");

        let g_hex = "f7e1a085d69b3ddecbbcab5c36b857b97994afbbfa3aea82f9574c0b3d0782675159578ebad4594fe67107108180b449167123e84c281613b7cf09328cc8a6e13c167a8b547c8d28e0a3ae1e2bb3a675916ea37f0bfa213562f1fb627a01243bcca4f1bea8519089a883dfe15ae59f06928b665e807b552564014c3bfecf492a02";
        let g = BigUint::parse_bytes(g_hex.as_bytes(), 16).expect("Invalid G hex");

        let x_hex = "358b1b91aa482f35bc5617c6ad4e1e1e8ebf0eac";
        let x = BigUint::parse_bytes(x_hex.as_bytes(), 16).expect("Invalid X hex");

        DsaPrivateKey { p, q, g, x }
    }

    pub fn sign_raw(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let hash = hasher.finalize();
        self.sign_hash(&hash)
    }

    fn sign_hash(&self, hash: &[u8]) -> Vec<u8> {
        let hash_int = BigUint::from_bytes_be(hash);
        let mut rng = thread_rng();

        loop {
            let k = rng.gen_biguint_range(&BigUint::from(2u32), &self.q);

            let r = self.g.modpow(&k, &self.p) % &self.q;

            if r == BigUint::from(0u32) {
                continue;
            }

            let k_inv = mod_inverse(&k, &self.q);
            match k_inv {
                Some(ki) => {
                    let xr = (&self.x * &r) % &self.q;
                    let hash_mod_q = &hash_int % &self.q;
                    let s = (ki * (&hash_mod_q + &xr)) % &self.q;

                    if s == BigUint::from(0u32) {
                        continue;
                    }

                    return encode_dsa_signature(&r, &s);
                }
                None => continue,
            }
        }
    }
}

fn mod_inverse(a: &BigUint, m: &BigUint) -> Option<BigUint> {
    let a_bi = BigInt::from_biguint(Sign::Plus, a.clone());
    let m_bi = BigInt::from_biguint(Sign::Plus, m.clone());

    let (g, x, _) = extended_gcd(&a_bi, &m_bi);

    if g != BigInt::from(1u32) {
        return None;
    }

    let result = ((x % &m_bi) + &m_bi) % &m_bi;
    let (_, result_bytes) = result.to_bytes_be();
    Some(BigUint::from_bytes_be(&result_bytes))
}

fn extended_gcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    if *b == BigInt::from(0u32) {
        return (a.clone(), BigInt::from(1u32), BigInt::from(0u32));
    }

    let (g, x1, y1) = extended_gcd(b, &(a % b));
    let x = y1.clone();
    let y = x1 - (a / b) * y1;
    (g, x, y)
}

fn encode_dsa_signature(r: &BigUint, s: &BigUint) -> Vec<u8> {
    let r_bytes = r.to_bytes_be();
    let s_bytes = s.to_bytes_be();

    let r_der = encode_der_integer(&r_bytes);
    let s_der = encode_der_integer(&s_bytes);

    let content = [r_der, s_der].concat();
    encode_der_sequence(&content)
}

fn encode_der_integer(bytes: &[u8]) -> Vec<u8> {
    let mut content = bytes.to_vec();
    if content[0] & 0x80 != 0 {
        content.insert(0, 0x00);
    }
    let mut result = vec![0x02]; // INTEGER tag
    result.push(content.len() as u8);
    result.extend(content);
    result
}

fn encode_der_sequence(content: &[u8]) -> Vec<u8> {
    let mut result = vec![0x30]; // SEQUENCE tag
    if content.len() > 127 {
        result.push(0x81);
        result.push(content.len() as u8);
    } else {
        result.push(content.len() as u8);
    }
    result.extend(content);
    result
}

fn zip_text(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).map_err(|e| format!("zlib write error: {}", e))?;
    encoder.finish().map_err(|e| format!("zlib finish error: {}", e))
}

fn write_int32(val: u32) -> Vec<u8> {
    vec![
        (val >> 24) as u8,
        (val >> 16) as u8,
        (val >> 8) as u8,
        val as u8,
    ]
}

fn split_lines(data: &str) -> String {
    let mut result = String::new();
    for (i, c) in data.chars().enumerate() {
        result.push(c);
        if (i + 1) % 76 == 0 && i + 1 < data.len() {
            result.push('\n');
        }
    }
    result
}

fn to_base31(value: u32) -> String {
    if value == 0 {
        return "0".to_string();
    }
    let chars: Vec<char> = "0123456789abcdefghijklmnopqrstu".chars().collect();
    let mut result = String::new();
    let mut v = value;
    while v > 0 {
        result.push(chars[(v % 31) as usize]);
        v /= 31;
    }
    result.chars().rev().collect()
}

    pub fn sign_data(data: &[u8]) -> Vec<u8> {
        let key = DsaPrivateKey::new();
        let mut hasher = Sha1::new();
        hasher.update(data);
        let hash = hasher.finalize();
        key.sign_hash(&hash)
    }

pub fn encode_license(license_text: &str) -> Result<String, String> {
    let key = DsaPrivateKey::new();
    
    let compressed = zip_text(license_text.as_bytes())?;

    let mut text = vec![13, 14, 12, 10, 15];
    text.extend(&compressed);

    let signature = key.sign_raw(&text);

    let mut out = write_int32(text.len() as u32);
    out.extend(&text);
    out.extend(&signature);

    let encoded = general_purpose::STANDARD.encode(&out);

    let result = format!("{}X02{}", encoded, to_base31(encoded.len() as u32));

    Ok(split_lines(&result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_initialization() {
        let key = DsaPrivateKey::new();
        assert!(key.p.bits() >= 1020, "P should be ~1024 bits");
        assert!(key.q.bits() >= 155, "Q should be ~160 bits");
    }

    #[test]
    fn test_mod_inverse() {
        let a = BigUint::from(3u32);
        let m = BigUint::from(7u32);
        let inv = mod_inverse(&a, &m).unwrap();
        assert_eq!(inv, BigUint::from(5u32), "3^-1 mod 7 should be 5");
    }

    #[test]
    fn test_zip_unzip_roundtrip() {
        let data = b"Hello, Atlassian!";
        let zipped = zip_text(data).unwrap();
        assert!(!zipped.is_empty());
    }

    #[test]
    fn test_signature_not_empty() {
        let key = DsaPrivateKey::new();
        let hash = [0xABu8; 20];
        let sig = key.sign_raw(&hash);
        assert!(!sig.is_empty(), "Signature should not be empty");
        assert!(sig.len() > 10, "Signature should contain r+s at least");
    }


    #[test]
    fn test_dsa_key_matches_java_reference() {
        let key = DsaPrivateKey::new();
        let p_expected = BigUint::parse_bytes(
            b"fd7f53811d75122952df4a9c2eece4e7f611b7523cef4400c31e3f80b6512669455d402251fb593d8d58fabfc5f5ba30f6cb9b556cd7813b801d346ff26660bb6b9950a5a49f9fe8047b1022c24fbba9d7feb7c61bf83b57e7c6a8a6150f04fb83f6d3c51ec3023554135a169132f675f3ae2b61d72aeff22203199dd14801c7",
            16,
        ).unwrap();
        assert_eq!(key.p, p_expected, "DSA P parameter mismatch vs Java reference");

        let q_expected = BigUint::parse_bytes(
            b"009760508f15230bccb292b982a2ee102fc16073d4",
            16,
        ).unwrap();
        assert_eq!(key.q, q_expected, "DSA Q parameter mismatch vs Java reference");

        let g_expected = BigUint::parse_bytes(
            b"f7e1a085d69b3ddecbbcab5c36b857b97994afbbfa3aea82f9574c0b3d0782675159578ebad4594fe67107108180b449167123e84c281613b7cf09328cc8a6e13c167a8b547c8d28e0a3ae1e2bb3a675916ea37f0bfa213562f1fb627a01243bcca4f1bea8519089a883dfe15ae59f06928b665e807b552564014c3bfecf492a02",
            16,
        ).unwrap();
        assert_eq!(key.g, g_expected, "DSA G parameter mismatch vs Java reference");

        let x_expected = BigUint::parse_bytes(
            b"358b1b91aa482f35bc5617c6ad4e1e1e8ebf0eac",
            16,
        ).unwrap();
        assert_eq!(key.x, x_expected, "DSA X parameter mismatch vs Java reference");
    }

    #[test]
    fn test_der_encoding_format() {
        let r = BigUint::from_bytes_be(&[0x12, 0x34, 0x56, 0x78]);
        let s = BigUint::from_bytes_be(&[0x9a, 0xbc, 0xde, 0xf0]);
        let encoded = encode_dsa_signature(&r, &s);

        assert_eq!(encoded[0], 0x30, "DER signature must start with SEQUENCE tag");

        let content_len = encoded[1] as usize;
        let content = &encoded[2..2 + content_len];

        assert_eq!(content[0], 0x02, "First element must be INTEGER tag");
        let r_len = content[1] as usize;
        let r_val = &content[2..2 + r_len];
        assert_eq!(r_val, &[0x12, 0x34, 0x56, 0x78], "r value mismatch");

        let s_start = 2 + r_len;
        assert_eq!(content[s_start], 0x02, "Second element must be INTEGER tag");
        let s_len = content[s_start + 1] as usize;
        let s_val = &content[s_start + 2..s_start + 2 + s_len];
        assert_eq!(s_val, &[0x00, 0x9a, 0xbc, 0xde, 0xf0], "s value with leading zero due to high bit");
    }

    #[test]
    fn test_der_integer_leading_zero() {
        let bytes = vec![0x80, 0x00, 0x00, 0x00];
        let der = encode_der_integer(&bytes);
        assert_eq!(der[0], 0x02, "INTEGER tag");
        assert_eq!(der[1], 5, "Length should be 5 due to leading zero");
        assert_eq!(der[2], 0x00, "Leading zero byte");
        assert_eq!(&der[3..], &bytes[..], "Original bytes should follow");

        let bytes2 = vec![0x7f, 0xff, 0xff, 0xff];
        let der2 = encode_der_integer(&bytes2);
        assert_eq!(der2[1], 4, "Length should be 4 without leading zero");
        assert_eq!(&der2[2..], &bytes2[..], "Original bytes without leading zero");
    }

    #[test]
    fn test_der_sequence_length_encoding() {
        let short_content = vec![0x02, 0x01, 0x00];
        let seq = encode_der_sequence(&short_content);
        assert_eq!(seq[0], 0x30);
        assert_eq!(seq[1], 3, "Short form length");

        let empty: Vec<u8> = vec![];
        let seq_empty = encode_der_sequence(&empty);
        assert_eq!(seq_empty, vec![0x30, 0x00]);
    }

    #[test]
    fn test_signature_der_structure() {
        let key = DsaPrivateKey::new();
        let hash = [0xABu8; 20];
        let sig = key.sign_raw(&hash);

        assert_eq!(sig[0], 0x30, "Signature must start with SEQUENCE");

        let content_len = sig[1] as usize;
        assert!(content_len + 2 <= sig.len(), "Content length must fit");

        let content = &sig[2..2 + content_len];

        assert_eq!(content[0], 0x02, "First element must be INTEGER");
        let r_len = content[1] as usize;
        assert!(r_len > 0 && r_len < 30, "r length should be reasonable");

        let s_offset = 2 + r_len;
        assert!(s_offset < content.len(), "s must fit in content");
        assert_eq!(content[s_offset], 0x02, "Second element must be INTEGER");
        let s_len = content[s_offset + 1] as usize;
        assert!(s_len > 0 && s_len < 30, "s length should be reasonable");

        assert!(r_len <= 21, "r should be <= 21 bytes DER-encoded");
        assert!(s_len <= 21, "s should be <= 21 bytes DER-encoded");
    }

    #[test]
    fn test_write_int32() {
        let result = write_int32(0x12345678);
        assert_eq!(result, vec![0x12, 0x34, 0x56, 0x78], "Big-endian int32");
        assert_eq!(write_int32(0), vec![0x00, 0x00, 0x00, 0x00], "Zero int32");
        assert_eq!(write_int32(1), vec![0x00, 0x00, 0x00, 0x01], "One int32");
        assert_eq!(write_int32(0xFFFFFFFF), vec![0xFF, 0xFF, 0xFF, 0xFF], "Max uint32");
    }

    #[test]
    fn test_zip_text_is_valid_zlib() {
        let data = b"Test license data for compression";
        let zipped = zip_text(data).unwrap();

        assert!(zipped.len() >= 6, "Zlib output must have at least header+checksum");

        assert_eq!(zipped[0] & 0x0F, 0x08, "Zlib: must be deflate method (CM=8)");

        use flate2::read::ZlibDecoder;
        use std::io::Read;
        let mut decoder = ZlibDecoder::new(&zipped[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).expect("Should decompress");
        assert_eq!(decompressed, data, "Round-trip compression should preserve data");
    }

    #[test]
    fn test_encode_license_format() {
        let license_text = "TestLicense";
        let result = encode_license(license_text).unwrap();

        assert!(!result.is_empty(), "License should not be empty");

        assert!(result.contains("X02"), "License must contain X02 suffix separator");

        let no_newlines = result.replace('\n', "");
        let newline_count = result.matches('\n').count();
        assert_eq!(result.len(), no_newlines.len() + newline_count,
            "Each \\n adds 1 to length");

        let lines: Vec<&str> = result.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if i < lines.len() - 1 {
                assert!(line.len() == 76, "Line {} should be exactly 76 chars, got {}", i, line.len());
            } else {
                assert!(line.len() <= 76, "Last line should be <= 76 chars");
            }
        }

        let x02_pos = no_newlines.rfind("X02").unwrap();
        let suffix = &no_newlines[x02_pos + 3..];
        assert!(!suffix.is_empty(), "Suffix must not be empty");
        for c in suffix.chars() {
            assert!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'u'),
                "Suffix char '{}' must be valid base-31 digit", c);
        }
    }

    #[test]
    fn test_encode_license_structural_components() {
        let license_text = "key1=value1\nkey2=value2";
        let result = encode_license(license_text).unwrap();
        let clean = result.replace('\n', "");

        let x02_pos = clean.rfind("X02").unwrap();
        let b64_part = &clean[..x02_pos];
        use base64::Engine;
        let decoded = general_purpose::STANDARD.decode(b64_part).expect("Base64 part must be valid");

        let text_len = u32::from_be_bytes([decoded[0], decoded[1], decoded[2], decoded[3]]);
        assert!(text_len > 0, "Length should be positive");

        assert_eq!(decoded[4], 13, "Magic byte 1");
        assert_eq!(decoded[5], 14, "Magic byte 2");
        assert_eq!(decoded[6], 12, "Magic byte 3");
        assert_eq!(decoded[7], 10, "Magic byte 4");
        assert_eq!(decoded[8], 15, "Magic byte 5");

        let compressed = &decoded[9..9 + text_len as usize];

        use flate2::read::ZlibDecoder;
        use std::io::Read;
        let mut decoder = ZlibDecoder::new(compressed);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).expect("Should decompress");
        assert_eq!(decompressed, license_text, "Should decompress to original text");

        let sig_start = 4 + text_len as usize;
        let sig_len = decoded.len() - sig_start;
        assert!(sig_len > 0, "Signature must be present (sig_len={})", sig_len);
        let signature = &decoded[sig_start..];

        assert_eq!(signature[0], 0x30, "Signature must be DER SEQUENCE");
    }

    #[test]
    fn test_magic_bytes_sequence() {
        let result = encode_license("any data").unwrap();
        let clean = result.replace('\n', "");
        let x02_pos = clean.rfind("X02").unwrap();
        let b64_part = &clean[..x02_pos];
        let decoded = general_purpose::STANDARD.decode(b64_part).unwrap();

        assert_eq!(decoded[4..9], vec![13, 14, 12, 10, 15],
            "Magic bytes must be [13, 14, 12, 10, 15] matching Java reference");
    }

    #[test]
    fn test_split_lines_76_chars() {
        let short = "short";
        assert_eq!(split_lines(short), "short", "Short string should not be broken");

        let exact = "a".repeat(76);
        assert_eq!(split_lines(&exact), exact, "76-char string should not be broken");

        let one_over = "a".repeat(78);
        let split = split_lines(&one_over);
        assert_eq!(split.len(), 79, "78 chars + 1 newline = 79");
        assert!(split.contains('\n'), "Should contain newline");
        let lines: Vec<&str> = split.lines().collect();
        assert_eq!(lines.len(), 2, "Should be split into 2 lines");
        assert_eq!(lines[0].len(), 76, "First line should be 76 chars");
        assert_eq!(lines[1].len(), 2, "Second line should be 2 chars");
    }

    #[test]
    fn test_to_base31_matches_java() {
        assert_eq!(to_base31(0), "0", "0 in base-31 should be '0'");

        assert_eq!(to_base31(31), "10", "31 in base-31 should be '10'");

        assert_eq!(to_base31(100), "37", "100 in base-31 should be '37'");

        assert_eq!(to_base31(12345), "cq7", "12345 in base-31 should be 'cq7'");

        assert_eq!(to_base31(1), "1", "1 in base-31 should be '1'");

        for val in [0u32, 1, 10, 31, 100, 255, 1000, 12345, 99999, 1000000, 0xFFFFFFFF] {
            let b31 = to_base31(val);
            let parsed = u32::from_str_radix(&b31, 31)
                .unwrap_or_else(|_| panic!("Should parse '{}' as base-31 number", b31));
            assert_eq!(parsed, val, "Roundtrip failed for {}", val);
        }
    }

    #[test]
    fn test_suffix_is_base31_not_hex() {
        let license_text = "Test suffix format";
        let result = encode_license(license_text).unwrap();
        let no_newlines = result.replace('\n', "");
        let x02_pos = no_newlines.rfind("X02").unwrap();
        let suffix = &no_newlines[x02_pos + 3..];

        for c in suffix.chars() {
            if c >= '0' && c <= '9' { continue; }
            if c >= 'a' && c <= 'u' { continue; }
            panic!("Invalid base-31 character '{}' in suffix '{}'", c, suffix);
        }

        let encoded_len = no_newlines[..x02_pos].len() as u32;
        assert_eq!(suffix, to_base31(encoded_len),
            "Suffix should be base-31 encoding of encoded length");
    }

    #[test]
    fn test_dsa_signature_length_range() {
        let key = DsaPrivateKey::new();
        let data = b"test data for signing";

        for _ in 0..10 {
            let sig = key.sign_raw(data);
            assert!(sig.len() >= 40, "Signature should be at least 40 bytes, got {}", sig.len());
            assert!(sig.len() <= 48, "Signature should be at most 48 bytes, got {}", sig.len());
        }
    }

    #[test]
    fn test_sign_data_public() {
        let data = b"Hello, World!";
        let sig = sign_data(data);
        assert!(!sig.is_empty(), "sign_data should produce a signature");
        assert_eq!(sig[0], 0x30, "sign_data should produce DER SEQUENCE");
    }
}