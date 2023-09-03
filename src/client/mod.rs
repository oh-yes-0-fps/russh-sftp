use std::{error::Error, sync::Arc};

use bytes::Bytes;
use russh::client::Config;
use russh_keys::key;
use tokio::io::AsyncWriteExt;

use crate::protocol::{Packet, types::{Open, OpenFlags}};

pub mod handler;
pub mod session;

pub struct Client {}

#[async_trait::async_trait]
impl russh::client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        self,
        _server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        Ok((self, true))
    }
}


pub async fn start_sftp_client(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn Error>> {
    let client = Client {};

    let mut configs = Config::default();
    configs.maximum_packet_size = 65535;
    configs.window_size = 65535;

    let configs = Arc::new(configs);

    let mut handle = russh::client::connect(
        configs,
        (host, port),
        client,
    ).await?;

    if !handle.authenticate_password(username, password).await? {
        Err(russh::Error::NotAuthenticated)?;
    };

    let mut channel = handle.channel_open_session().await?;

    // request sftp subsystem
    channel.request_subsystem(false, "sftp").await?;

    channel.wait().await;

    let create_file = Packet::Open(
        Open {
            filename: "~/test.txt".to_string(),
            pflags: OpenFlags::from_bits_retain(0x0000000a),
            attrs: Default::default(),
            id: 0,
        }
    );

    #[cfg(test)]
    {
        
    }

    let mut stream = channel.into_stream();

    let mut bytes: Bytes = create_file.try_into().unwrap();
    stream.write_all(&mut bytes).await.unwrap();
    stream.flush().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // let mut bytes = BytesMut::new();
    // stream.read_buf(&mut bytes).await.unwrap();

    // let packet = Packet::try_from(&mut bytes.freeze()).unwrap();
    // info!("packet: {:?}", packet);

    // info!("bytes: {:?}", bytes);
    // stream.write_u8(0).await.unwrap();
    Ok(())
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_sftp() {
        env_logger::
            builder().
            is_test(true).
            filter_level(log::LevelFilter::Info).
            init();
        // use tokio::io::{AsyncRead, AsyncWrite};

        let host = "localhost";
        let port = 2222;
        let username: &str = "";
        let password: &str = "";

        super::start_sftp_client(host, port, username, password).await.unwrap();

        // sftp.stream.
    }
}