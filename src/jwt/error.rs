use jsonwebtoken::errors::Error as JwtError;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Encoding to jwt failed")]
pub struct EncodingError(#[from] JwtError);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Decoding to jwt failed")]
pub struct DecodingError(#[from] JwtError);

#[derive(Debug, PartialEq, Eq, Hash, thiserror::Error)]
#[error("Unssuported algorithm provided")]
pub struct UnsupportedAlgorithm(());

impl UnsupportedAlgorithm {
    pub(super) fn new() -> Self {
        Self(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecoderError {
    #[error("Loading PEM failed")]
    LoadKeyFailed(#[source] JwtError),
    #[error("Decoding JWT header failed")]
    DecodeHeaderFailed(#[source] JwtError),
    #[error("Decoding JWT payload failed")]
    DecodePayloadFailed(#[source] JwtError),
    #[error(transparent)]
    UnsupportedAlgorithm(UnsupportedAlgorithm),
}

impl DecoderError {
    pub(super) fn unsupported_alg() -> Self {
        Self::UnsupportedAlgorithm(UnsupportedAlgorithm::new())
    }
}
