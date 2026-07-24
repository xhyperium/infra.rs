//! 证据签名（HMAC-SHA256，基于 workspace `sha2`；无额外 hmac crate）。

use sha2::{Digest, Sha256};

use crate::EvidenceError;

/// 带签名的 wire 记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedEvidence {
    /// 序号。
    pub seq: u64,
    /// 事件名。
    pub name: String,
    /// HMAC-SHA256 摘要（32 字节）。
    pub signature: [u8; 32],
}

/// 用密钥对 `(seq, name)` 做 HMAC-SHA256。
#[must_use]
pub fn sign_evidence(key: &[u8], seq: u64, name: &str) -> SignedEvidence {
    let signature = hmac_sha256(key, &canonical_bytes(seq, name));
    SignedEvidence { seq, name: name.to_string(), signature }
}

/// 校验签名；失败返回 [`EvidenceError::DurabilityFailure`]。
pub fn verify_evidence(key: &[u8], record: &SignedEvidence) -> Result<(), EvidenceError> {
    let expected = hmac_sha256(key, &canonical_bytes(record.seq, &record.name));
    if constant_time_eq(&expected, &record.signature) {
        Ok(())
    } else {
        Err(EvidenceError::DurabilityFailure)
    }
}

/// 规范载荷：`seq` 十进制 ASCII + `\t` + name（UTF-8）。
#[must_use]
pub fn canonical_bytes(seq: u64, name: &str) -> Vec<u8> {
    let mut v = seq.to_string().into_bytes();
    v.push(b'\t');
    v.extend_from_slice(name.as_bytes());
    v
}

/// 十六进制编码（小写），便于日志/wire 文本。
#[must_use]
pub fn signature_hex(sig: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for b in sig {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

/// HMAC-SHA256（RFC 2104）。
#[must_use]
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut key_block = [0u8; BLOCK];
    if key.len() > BLOCK {
        let dig = Sha256::digest(key);
        key_block[..32].copy_from_slice(&dig);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; BLOCK];
    let mut opad = [0x5cu8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let inner_hash = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    let out = outer.finalize();
    let mut sig = [0u8; 32];
    sig.copy_from_slice(&out);
    sig
}

fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify_roundtrip() {
        let key = b"test-key";
        let rec = sign_evidence(key, 7, "evt");
        verify_evidence(key, &rec).unwrap();
        let mut bad = rec.clone();
        bad.signature[0] ^= 0xff;
        assert_eq!(verify_evidence(key, &bad), Err(EvidenceError::DurabilityFailure));
        assert_eq!(signature_hex(&rec.signature).len(), 64);
        // long key path
        let long = vec![0xabu8; 80];
        let r2 = sign_evidence(&long, 1, "x");
        verify_evidence(&long, &r2).unwrap();
    }
}
