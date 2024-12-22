//! Shorten UUID

use std::fmt;

use uuid::Uuid;

const IGNORE_BITS: u128 = 40;

/// Shorten token from UUID
///
/// This uses just head 11 bytes of 16 bytes in UUID.
/// It should suffice in a *tiny* service.
#[derive(Debug, Clone, Copy)]
pub(super) struct Token(Uuid);

impl From<Uuid> for Token {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl Token {
    pub(super) fn new(id: Uuid) -> Self {
        Self(id)
    }

    pub(super) fn as_alphanumeric(&self) -> Alphanumeric<'_> {
        Alphanumeric(self)
    }

    pub(super) fn as_hex(&self) -> Hex<'_> {
        Hex(self)
    }

    fn as_bytes(&self) -> &uuid::Bytes {
        self.0.as_bytes()
    }

    fn as_u128(&self) -> u128 {
        self.0.as_u128()
    }

    pub(super) fn parse_alphanumeric(value: &str) -> Result<Self, Error> {
        let (res, len) = value.chars().try_fold((0u128, 0usize), |(v, l), c| {
            let codepoint = u8::try_from(c)?;
            let digit = Alphanumeric::try_codepoint_to_digit(codepoint)?;
            let v = v * 62 + digit as u128;
            let l = l + 1;
            Ok::<_, Error>((v, l))
        })?;
        if len != 16 {
            return Err(Error::UnexpectedLength);
        }
        let uuid = uuid::Builder::from_u128(res << IGNORE_BITS).into_uuid();
        Ok(Self(uuid))
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Alphanumeric<'a>(&'a Token);

pub const ALPHANUMERIC_STR_LENGTH: usize = 16;

impl Alphanumeric<'_> {
    fn digit_to_codepoint(digit: u8) -> u8 {
        if digit < 10 {
            digit + b'0'
        } else if digit < 36 {
            digit - 10 + b'a'
        } else {
            digit - 36 + b'A'
        }
    }

    fn try_codepoint_to_digit(codepoint: u8) -> Result<u8, Error> {
        if (b'0'..b'0' + 10).contains(&codepoint) {
            Ok(codepoint - b'0')
        } else if (b'a'..b'a' + 26).contains(&codepoint) {
            Ok(codepoint - b'a' + 10)
        } else if (b'A'..b'A' + 26).contains(&codepoint) {
            Ok(codepoint - b'A' + 36)
        } else {
            Err(Error::NonAlphanumeric)
        }
    }

    pub fn encode_buffer() -> [u8; ALPHANUMERIC_STR_LENGTH] {
        [b'0'; ALPHANUMERIC_STR_LENGTH]
    }

    pub fn encode<'buf>(&self, buffer: &'buf mut [u8; ALPHANUMERIC_STR_LENGTH]) -> &'buf str {
        let mut v = self.0.as_u128() >> IGNORE_BITS;
        for i in (0usize..16).rev() {
            let rem = (v % 62) as u8;
            v /= 62;
            buffer[i] = Self::digit_to_codepoint(rem);
        }
        std::str::from_utf8(buffer).unwrap()
    }
}

impl fmt::Display for Alphanumeric<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = Self::encode_buffer();
        let s = self.encode(&mut buf);
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Hex<'a>(&'a Token);

impl Hex<'_> {
    fn word_to_codepoint(word: u8) -> u8 {
        if (0..0xa).contains(&word) {
            word + b'0'
        } else {
            // (0xa..=0xf).contains(word)
            word - 10 + b'a'
        }
    }

    /// digit -> (upper, lower)
    fn digit_to_codepoints(digit: u8) -> (u8, u8) {
        let u = Self::word_to_codepoint(digit >> 4);
        let l = Self::word_to_codepoint(digit & 0x0f);
        (u, l)
    }
}

impl fmt::Display for Hex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const BYTES_CUT: usize = 16 - (IGNORE_BITS as usize / 8);
        let bytes = self.0.as_bytes();
        let bytes = &bytes[0..BYTES_CUT];
        let mut encode_buf = [0u8; BYTES_CUT * 2];
        for (i, d) in bytes.iter().copied().enumerate() {
            let (upper, lower) = Self::digit_to_codepoints(d);
            encode_buf[i * 2] = upper;
            encode_buf[i * 2 + 1] = lower;
        }
        let s = std::str::from_utf8(&encode_buf).unwrap();
        f.write_str(s)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected length of str received")]
    UnexpectedLength,
    #[error("Non-alphanumeric character found")]
    NonAlphanumeric,
    #[error("Non-ascii character found")]
    NonAscii(#[from] std::char::TryFromCharError),
}
