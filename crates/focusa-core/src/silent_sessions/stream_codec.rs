use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::SilentSessionRunId;

pub const STREAM_CHUNK_CODEC_VERSION: u32 = 1;
pub const STREAM_CURSOR_VERSION: u32 = 1;
const CHUNK_MAGIC: &[u8; 4] = b"FSS1";
const MAX_DECOMPRESSED_CHUNK_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StreamCodecError {
    #[error("invalid compressed chunk header")]
    InvalidHeader,
    #[error("compressed chunk is truncated")]
    Truncated,
    #[error("decompressed chunk exceeds declared or maximum size")]
    SizeMismatch,
    #[error("invalid stream cursor")]
    InvalidCursor,
    #[error("unsupported stream cursor version: {0}")]
    UnsupportedCursorVersion(u32),
    #[error("stream cursor checksum mismatch")]
    CursorChecksumMismatch,
}

pub fn compress_chunk(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len() + 12);
    output.extend_from_slice(CHUNK_MAGIC);
    output.extend_from_slice(&(input.len() as u64).to_be_bytes());
    let mut index = 0;
    while index < input.len() {
        let run = repeated_len(input, index);
        if run >= 4 {
            output.push(0x80 | ((run - 1) as u8));
            output.push(input[index]);
            index += run;
            continue;
        }
        let start = index;
        index += run;
        while index < input.len() && index - start < 128 {
            let next_run = repeated_len(input, index);
            if next_run >= 4 || index - start + next_run > 128 {
                break;
            }
            index += next_run;
        }
        let literal_len = index - start;
        output.push((literal_len - 1) as u8);
        output.extend_from_slice(&input[start..index]);
    }
    output
}

pub fn decompress_chunk(input: &[u8]) -> Result<Vec<u8>, StreamCodecError> {
    if input.len() < 12 || &input[..4] != CHUNK_MAGIC {
        return Err(StreamCodecError::InvalidHeader);
    }
    let declared = u64::from_be_bytes(
        input[4..12]
            .try_into()
            .map_err(|_| StreamCodecError::InvalidHeader)?,
    ) as usize;
    if declared > MAX_DECOMPRESSED_CHUNK_BYTES {
        return Err(StreamCodecError::SizeMismatch);
    }
    let mut output = Vec::with_capacity(declared);
    let mut index = 12;
    while index < input.len() {
        let control = input[index];
        index += 1;
        let len = ((control & 0x7f) as usize) + 1;
        if control & 0x80 != 0 {
            let byte = *input.get(index).ok_or(StreamCodecError::Truncated)?;
            index += 1;
            output.extend(std::iter::repeat_n(byte, len));
        } else {
            let end = index
                .checked_add(len)
                .ok_or(StreamCodecError::SizeMismatch)?;
            let literal = input.get(index..end).ok_or(StreamCodecError::Truncated)?;
            output.extend_from_slice(literal);
            index = end;
        }
        if output.len() > declared {
            return Err(StreamCodecError::SizeMismatch);
        }
    }
    if output.len() != declared {
        return Err(StreamCodecError::SizeMismatch);
    }
    Ok(output)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamCursor {
    pub version: u32,
    pub run_id: SilentSessionRunId,
    pub sequence: u64,
}

impl StreamCursor {
    pub fn new(run_id: SilentSessionRunId, sequence: u64) -> Self {
        Self {
            version: STREAM_CURSOR_VERSION,
            run_id,
            sequence,
        }
    }

    pub fn encode(self) -> Result<String, StreamCodecError> {
        let body = serde_json::to_vec(&self).map_err(|_| StreamCodecError::InvalidCursor)?;
        let checksum = Sha256::digest(&body);
        let mut envelope = Vec::with_capacity(body.len() + checksum.len());
        envelope.extend_from_slice(&body);
        envelope.extend_from_slice(&checksum);
        Ok(URL_SAFE_NO_PAD.encode(envelope))
    }

    pub fn decode(encoded: &str) -> Result<Self, StreamCodecError> {
        let envelope = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| StreamCodecError::InvalidCursor)?;
        if envelope.len() <= 32 {
            return Err(StreamCodecError::InvalidCursor);
        }
        let split = envelope.len() - 32;
        let (body, checksum) = envelope.split_at(split);
        if Sha256::digest(body).as_slice() != checksum {
            return Err(StreamCodecError::CursorChecksumMismatch);
        }
        let cursor: Self =
            serde_json::from_slice(body).map_err(|_| StreamCodecError::InvalidCursor)?;
        if cursor.version != STREAM_CURSOR_VERSION {
            return Err(StreamCodecError::UnsupportedCursorVersion(cursor.version));
        }
        Ok(cursor)
    }
}

fn repeated_len(input: &[u8], start: usize) -> usize {
    let byte = input[start];
    let mut len = 1;
    while start + len < input.len() && input[start + len] == byte && len < 128 {
        len += 1;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_codec_roundtrips_mixed_bytes() {
        let input = b"aaaaaaaabbbb semantic output 12345 zzzzzzzzzzz";
        let compressed = compress_chunk(input);
        assert_eq!(decompress_chunk(&compressed).unwrap(), input);
    }

    #[test]
    fn chunk_codec_rejects_corruption_and_oversize() {
        assert_eq!(
            decompress_chunk(b"invalid"),
            Err(StreamCodecError::InvalidHeader)
        );
        let mut encoded = compress_chunk(b"proof");
        encoded.pop();
        assert!(decompress_chunk(&encoded).is_err());
    }

    #[test]
    fn cursor_roundtrip_and_tamper_detection() {
        let cursor = StreamCursor::new(SilentSessionRunId::new(), 42);
        let encoded = cursor.encode().unwrap();
        assert_eq!(StreamCursor::decode(&encoded).unwrap(), cursor);
        let mut tampered = encoded.into_bytes();
        let last = tampered.len() - 1;
        tampered[last] = if tampered[last] == b'A' { b'B' } else { b'A' };
        assert!(StreamCursor::decode(std::str::from_utf8(&tampered).unwrap()).is_err());
    }
}
