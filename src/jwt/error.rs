use jsonwebtoken::errors::Error as JwtError;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Encoding to jwt failed")]
pub struct EncodingError(#[from] JwtError);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Decoding to jwt failed")]
pub struct DecodingError(#[from] JwtError);
