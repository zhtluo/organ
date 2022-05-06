use async_std::io::{ReadExt as AsyncRead, WriteExt as AsyncWrite};
use std::io::{Read, Write};
use std::net::TcpStream;

// Network helpers to wrap message.

/// Error during network operations.
#[derive(Debug)]
pub struct NetError;
impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Network Error")
    }
}
impl std::error::Error for NetError {}

/// Read from a stream.
pub fn read_stream(stream: &mut TcpStream) -> Result<Vec<u8>, NetError> {
    let mut len_buf: [u8; 8] = [0; 8];
    if stream.read_exact(&mut len_buf).is_err() {
        return Err(NetError {});
    }
    let len = u64::from_ne_bytes(len_buf);
    let mut buf = Vec::<u8>::with_capacity(len.try_into().unwrap());
    if stream.take(len).read_to_end(&mut buf).is_err() {
        return Err(NetError {});
    }
    Ok(buf)
}

/// Write to a stream.
pub fn write_stream(stream: &mut TcpStream, data: &[u8]) -> Result<(), NetError> {
    let len_buf = u64::to_ne_bytes(data.len().try_into().unwrap());
    if stream.write_all(&len_buf).is_err() {
        return Err(NetError {});
    }
    if stream.write_all(data).is_err() {
        return Err(NetError {});
    }
    if stream.flush().is_err() {
        return Err(NetError {});
    }
    Ok(())
}

/// Read from a stream.
pub async fn async_read_stream(
    stream: &mut async_std::net::TcpStream,
) -> Result<Vec<u8>, NetError> {
    let mut len_buf: [u8; 8] = [0; 8];
    if stream.read_exact(&mut len_buf).await.is_err() {
        return Err(NetError {});
    }
    let len = u64::from_ne_bytes(len_buf);
    let mut buf = Vec::<u8>::with_capacity(len.try_into().unwrap());
    if stream.take(len).read_to_end(&mut buf).await.is_err() {
        return Err(NetError {});
    }
    Ok(buf)
}

/// Write to a stream.
pub async fn async_write_stream(
    stream: &mut async_std::net::TcpStream,
    data: &[u8],
) -> Result<(), NetError> {
    let len_buf = u64::to_ne_bytes(data.len().try_into().unwrap());
    if stream.write_all(&len_buf).await.is_err() {
        return Err(NetError {});
    }
    if stream.write_all(data).await.is_err() {
        return Err(NetError {});
    }
    if stream.flush().await.is_err() {
        return Err(NetError {});
    }
    Ok(())
}
