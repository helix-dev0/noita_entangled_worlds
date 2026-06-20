use std::marker::PhantomData;

use bitcode::{DecodeOwned, Encode};
use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::trace;

use super::DirectConnectionError;
use crate::MAX_MESSAGE_LEN;

/// Reject lengths past MAX_MESSAGE_LEN so a bogus or malicious wire length
/// can't drive a multi-gigabyte allocation on receive or a panic on send.
fn validate_len(len: usize) -> Result<(), DirectConnectionError> {
    if len > MAX_MESSAGE_LEN {
        Err(DirectConnectionError::MessageTooLong(len))
    } else {
        Ok(())
    }
}

pub(crate) struct SendMessageStream<Msg> {
    inner: SendStream,
    _phantom: PhantomData<fn(Msg)>,
}

pub(crate) struct RecvMessageStream<Msg> {
    inner: RecvStream,
    _phantom: PhantomData<fn() -> Msg>,
}

impl<Msg: Encode> SendMessageStream<Msg> {
    pub(crate) fn new(inner: SendStream) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    async fn send_raw(&mut self, msg: &[u8]) -> Result<(), DirectConnectionError> {
        validate_len(msg.len())?;
        let len = u32::try_from(msg.len())
            .map_err(|_| DirectConnectionError::MessageTooLong(msg.len()))?;
        self.inner
            .write_u32(len)
            .await
            .map_err(|_err| DirectConnectionError::MessageIoFailed)?;
        self.inner
            .write_all(msg)
            .await
            .map_err(|_err| DirectConnectionError::MessageIoFailed)?;
        Ok(())
    }

    pub(crate) async fn send(&mut self, msg: &Msg) -> Result<(), DirectConnectionError> {
        let msg = bitcode::encode(msg);
        self.send_raw(&msg).await
    }
}

impl<Msg: DecodeOwned> RecvMessageStream<Msg> {
    pub(crate) fn new(inner: RecvStream) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    async fn recv_raw(&mut self) -> Result<Vec<u8>, DirectConnectionError> {
        let len = self
            .inner
            .read_u32()
            .await
            .map_err(|_err| DirectConnectionError::MessageIoFailed)? as usize;
        validate_len(len)?;
        trace!("Expecting message of len {len}");
        let mut buf = vec![0; len];
        self.inner
            .read_exact(&mut buf)
            .await
            .map_err(|_err| DirectConnectionError::MessageIoFailed)?;
        Ok(buf)
    }
    pub(crate) async fn recv(&mut self) -> Result<Msg, DirectConnectionError> {
        let raw = self.recv_raw().await?;
        bitcode::decode(&raw).map_err(|_| DirectConnectionError::DecodeError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_len_rejects_oversized() {
        assert!(validate_len(0).is_ok());
        assert!(validate_len(MAX_MESSAGE_LEN).is_ok());
        assert!(matches!(
            validate_len(MAX_MESSAGE_LEN + 1),
            Err(DirectConnectionError::MessageTooLong(_))
        ));
    }
}
