use async_std::io::{ReadExt as AsyncRead, WriteExt as AsyncWrite};
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn read_stream(stream: &mut TcpStream) -> Result<Vec<u8>, ()> {
    let mut len_buf: [u8; 8] = [0; 8];
    stream.read_exact(&mut len_buf).unwrap();
    let len = u64::from_ne_bytes(len_buf);
    let mut buf = Vec::<u8>::with_capacity(len.try_into().unwrap());
    stream.take(len).read_to_end(&mut buf).unwrap();
    Ok(buf)
}

pub fn write_stream(stream: &mut TcpStream, data: &Vec<u8>) -> Result<(), ()> {
    let len_buf = u64::to_ne_bytes(data.len().try_into().unwrap());
    stream.write(&len_buf).unwrap();
    stream.write(&data).unwrap();
    stream.flush().unwrap();
    Ok(())
}

pub async fn async_read_stream(stream: &mut async_std::net::TcpStream) -> Result<Vec<u8>, ()> {
    let mut len_buf: [u8; 8] = [0; 8];
    stream.read_exact(&mut len_buf).await.unwrap();
    let len = u64::from_ne_bytes(len_buf);
    let mut buf = Vec::<u8>::with_capacity(len.try_into().unwrap());
    stream.take(len).read_to_end(&mut buf).await.unwrap();
    Ok(buf)
}

pub async fn async_write_stream(
    stream: &mut async_std::net::TcpStream,
    data: &Vec<u8>,
) -> Result<(), ()> {
    let len_buf = u64::to_ne_bytes(data.len().try_into().unwrap());
    stream.write(&len_buf).await.unwrap();
    stream.write(&data).await.unwrap();
    stream.flush().await.unwrap();
    Ok(())
}
