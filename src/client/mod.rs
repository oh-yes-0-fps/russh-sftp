use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    error::Error,
    protocol::{Packet, RequestId, StatusCode}, handler_call,
};


mod handler;
pub use self::handler::Handler;
#[cfg(feature = "impls")]
pub mod implementation;


async fn read_buf<S>(stream: &mut S) -> Result<Bytes, Error>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let length = stream.read_u32().await?;

    let mut buf = vec![0; length as usize];
    stream.read_exact(&mut buf).await?;

    Ok(Bytes::from(buf))
}

async fn exec_response<H>(packet: Packet, processor: &mut H) -> Option<Packet>
where
    H: Handler + Send,
{
    Some(match packet {
        Packet::Version(version) => {
            let id = RequestId::get_request_id(&version);
            match processor.version(version).await {
                Err(err) => Packet::error(id, err.into()),
                Ok(_) => return None,
            }
        }
        Packet::Handle(handle) => handler_call!(processor, handle),
        Packet::Data(data) => handler_call!(processor, data),
        Packet::Name(name) => handler_call!(processor, name),
        Packet::Attrs(attrs) => handler_call!(processor, attrs),
        Packet::ExtendedReply(extended_reply) => handler_call!(processor, extended_reply),
        _ => Packet::error(0, StatusCode::BadMessage),
    })
}

async fn packet_processor<H, S>(stream: &mut S, handler: &mut H) -> Result<(), Error>
where
    H: Handler + Send,
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut bytes = read_buf(stream).await?;

    let request = match Packet::try_from(&mut bytes) {
        Ok(response) => exec_response(response, handler).await,
        Err(e) => {
            warn!("error: {:?}", e);
            Some(Packet::error(0, StatusCode::BadMessage))
        }
    };

    if request.is_none() {
        return Ok(());
    }

    let packet = Bytes::try_from(request.unwrap())?;
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