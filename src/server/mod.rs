mod handler;

#[cfg(feature = "impls")]
pub mod implementation;

use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    error::Error,
    protocol::{Packet, RequestId, StatusCode}, handler_call,
};

pub use self::handler::Handler;

async fn read_buf<S>(stream: &mut S) -> Result<Bytes, Error>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let length = stream.read_u32().await?;

    let mut buf = vec![0; length as usize];
    stream.read_exact(&mut buf).await?;

    Ok(Bytes::from(buf))
}

async fn exec_request<H>(packet: Packet, processor: &mut H) -> Packet
where
    H: Handler + Send,
{
    match packet {
        Packet::Init(init) => handler_call!(processor, init),
        Packet::Open(open) => handler_call!(processor, open),
        Packet::Close(close) => handler_call!(processor, close),
        Packet::Read(read) => handler_call!(processor, read),
        Packet::Write(write) => handler_call!(processor, write),
        Packet::LStat(lstat) => handler_call!(processor, lstat),
        Packet::FStat(fstat) => handler_call!(processor, fstat),
        Packet::SetStat(setstat) => handler_call!(processor, setstat),
        Packet::FSetStat(fsetstat) => handler_call!(processor, fsetstat),
        Packet::OpenDir(opendir) => handler_call!(processor, opendir),
        Packet::ReadDir(readdir) => handler_call!(processor, readdir),
        Packet::Remove(remove) => handler_call!(processor, remove),
        Packet::MkDir(mkdir) => handler_call!(processor, mkdir),
        Packet::RmDir(rmdir) => handler_call!(processor, rmdir),
        Packet::RealPath(realpath) => handler_call!(processor, realpath),
        Packet::Stat(stat) => handler_call!(processor, stat),
        Packet::Rename(rename) => handler_call!(processor, rename),
        Packet::ReadLink(readlink) => handler_call!(processor, readlink),
        Packet::Symlink(symlink) => handler_call!(processor, symlink),
        Packet::Extended(extended) => handler_call!(processor, extended),
        _ => Packet::error(0, StatusCode::BadMessage),
    }
}

async fn packet_processor<H, S>(stream: &mut S, handler: &mut H) -> Result<(), Error>
where
    H: Handler + Send,
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut bytes = read_buf(stream).await?;

    let response = match Packet::try_from(&mut bytes) {
        Ok(request) => exec_request(request, handler).await,
        Err(e) => {
            warn!("error: {:?}", e);
            Packet::error(0, StatusCode::BadMessage)
        }
    };

    let packet = Bytes::try_from(response)?;
    stream.write_all(&packet).await?;

    Ok(())
}

/// Run processing stream as SFTP
pub async fn run<S, H>(mut stream: S, mut handler: H)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    H: Handler + Send + 'static,
{
    tokio::spawn(async move {
        loop {
            match packet_processor(&mut stream, &mut handler).await {
                Err(Error::UnexpectedEof) => break,
                Err(err) => warn!("{}", err),
                Ok(_) => (),
            }
        }

        debug!("sftp stream ended");
    });
}
