use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespEncode, RespError};

use super::{extract_fixed_data, parse_length, parse_null, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct BulkString {
    pub(crate) data: Vec<u8>,
    pub(crate) is_null: bool,
}

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        if self.is_null {
            return b"$-1\r\n".to_vec();
        }
        let mut buf = Vec::with_capacity(self.data.len() as usize + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.data.len()).into_bytes());
        buf.extend_from_slice(&self.data);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if parse_null(buf, Self::PREFIX) {
            extract_fixed_data(buf, "$-1\r\n", "BulkString")?;
            return Ok(BulkString::new(vec![], true));
        }

        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec(), false))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        if parse_null(buf, Self::PREFIX) {
            return Ok(5);
        }
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>, is_null: bool) -> Self {
        BulkString { data: s.into(), is_null }
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString::new(s.as_bytes().to_vec(), false)
    }
}

impl From<String> for BulkString {
    fn from(s: String) -> Self {
        BulkString::new(s.into_bytes(), false)
    }
}

impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString::new(s.to_vec(), false)
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString::new(s.to_vec(), false)
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = BulkString::new(b"hello".to_vec(), false).into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_null_bulk_string_encode() {
        let frame: RespFrame = BulkString::new(b"".to_vec(), true).into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello", false));

        buf.extend_from_slice(b"$5\r\nhello");
        let ret = BulkString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello", false));

        Ok(())
    }

    #[test]
    fn test_null_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"", true));

        Ok(())
    }
}
