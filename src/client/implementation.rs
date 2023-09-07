use std::{collections::HashMap, io::SeekFrom, path::PathBuf};

use tokio::{io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt}, sync::Mutex};

use crate::{
    protocol::{types::*, Status, StatusCode, VERSION, Packet},
    sftp_fs::file::SftpFile,
};

use super::Handler;

#[derive(Debug, Default)]
pub struct SftpClientHandleImpl {
    remote_files: Mutex<HashMap<String, SftpFile>>,
    local_files: Mutex<HashMap<PathBuf, SftpFile>>,
    dir: Mutex<HashMap<PathBuf, String>>,
    outstanding_packets: Mutex<HashMap<u32, Packet>>
}


#[async_trait::async_trait]
impl Handler for SftpClientHandleImpl {
    type Error = StatusCode;

    fn unimplemented(&self) -> Self::Error {
        StatusCode::OpUnsupported
    }

    async fn version(&mut self, arg: Version) -> Result<(), Self::Error> {
        if arg.version != VERSION {
            return Err(self.unimplemented());
        }
        Ok(())
    }

    async fn handle(&mut self, arg: Handle) -> Result<Status, Self::Error> {
        let id = arg.id;
        if let Some(packet) = self.outstanding_packets.lock().await.remove(&id) {
            match packet {
                Packet::Open(open) => {
                    
                }
            }
        }
    }
}
