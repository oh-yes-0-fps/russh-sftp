use russh::{ChannelStream, ChannelMsg, ChannelId, Channel};

#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate serde;

mod buf;
/// Client side
pub mod client;
mod de;
mod error;
mod sftp_fs;
/// Protocol implementation
pub mod protocol;
mod ser;
/// Server side
pub mod server;
mod utils;

#[macro_export]
macro_rules! handler_call {
    ($handler:expr, $var:ident) => {
        {
            let id = RequestId::get_request_id(&$var);
            match $handler.$var($var).await {
                Err(err) => Packet::error(id, err.into()),
                Ok(packet) => packet.into(),
            }
        }
    };
}

trait ChannelSftpExt {
    fn into_multi_stream(self) -> (ChannelStream, ChannelStream);
}

impl<S: From<(ChannelId, ChannelMsg)> + Send + 'static> ChannelSftpExt for Channel<S> {
    fn into_multi_stream(mut self) -> (ChannelStream, ChannelStream) {
        let (runner_stream, mut r_rx, r_tx) = ChannelStream::new();
        let (session_stream, mut s_rx, s_tx) = ChannelStream::new();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    runner_msg = r_rx.recv() => {
                        match runner_msg {
                            Some(data) if !data.is_empty() => self.data(&data[..]).await?,
                            Some(_) => {
                                log::debug!("closing chan {:?}, received empty data", &self.id());
                                self.eof().await?;
                                self.close().await?;
                                break;
                            },
                            None => {
                                self.close().await?;
                                break
                            }
                        }
                    },
                    session_msg = s_rx.recv() => {
                        match session_msg {
                            Some(data) if !data.is_empty() => self.data(&data[..]).await?,
                            Some(_) => {
                                log::debug!("closing chan {:?}, received empty data", &self.id());
                                self.eof().await?;
                                self.close().await?;
                                break;
                            },
                            None => {
                                self.close().await?;
                                break
                            }
                        }
                    },
                    msg = self.wait() => {
                        match msg {
                            Some(ChannelMsg::Data { data }) => {
                                r_tx.send(data[..].into()).map_err(|_| russh::Error::SendError)?;
                            }
                            Some(ChannelMsg::Eof) => {
                                // Send a 0-length chunk to indicate EOF.
                                r_tx.send("".into()).map_err(|_| russh::Error::SendError)?;
                                s_tx.send("".into()).map_err(|_| russh::Error::SendError)?;
                                break
                            }
                            None => break,
                            _ => (),
                        }
                    }
                }
            }
            Ok::<_, russh::Error>(())
        });
        (runner_stream, session_stream)
    }
}