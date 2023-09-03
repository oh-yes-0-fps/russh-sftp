use std::{path::PathBuf,  pin::Pin, task::{Context, Poll}, io::{ErrorKind, Error, SeekFrom, Cursor}};

use tokio::{sync::mpsc::Sender, io::{AsyncWrite, AsyncRead, AsyncSeek, ReadBuf, self}, fs::File};

use crate::protocol::types::OpenFlags;

#[derive(Debug)]
struct RemoteFile {
    sender: Sender<Vec<u8>>,
    bytes: Cursor<Vec<u8>>
}

impl AsyncWrite for RemoteFile {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        self.sender.try_send(buf.to_vec()).map_err(|_| Error::new(ErrorKind::Other, "Failed to send data"))?;
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for RemoteFile {
    fn poll_read(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let bytes = self.get_mut().bytes.get_mut();
        let len = bytes.len();
        let cnt = std::cmp::min(len, buf.remaining());
        buf.put_slice(&bytes[..cnt]);
        Poll::Ready(Ok(()))
    }
}

impl AsyncSeek for RemoteFile {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> Result<(), Error> {
        let calc_pos = match position {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(pos) => self.bytes.get_ref().len() as i64 + pos,
            SeekFrom::Current(pos) => self.bytes.position() as i64 + pos,
        };
        self.bytes.set_position(calc_pos as u64);
        Ok(())
    }

    fn poll_complete(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<u64, Error>> {
        Poll::Ready(Ok(self.bytes.position()))
    }
}




#[derive(Debug)]
enum Owner {
    Client(RemoteFile),
    Server(File),
}

#[derive(Debug)]
enum Flags {
    ReadOnly,
    WriteOnly,
    AppendOnly,
    ReadWrite,
    ReadAppend,
}

impl From<OpenFlags> for Flags {
    fn from(flags: OpenFlags) -> Self {
        if flags.read() && flags.append() {
            return Flags::ReadAppend;
        }

        if flags.read() && flags.write() {
            return Flags::ReadWrite;
        }

        if flags.read() {
            return Flags::ReadOnly;
        }

        if flags.write() {
            return Flags::WriteOnly;
        }

        if flags.append() {
            return Flags::AppendOnly;
        }

        panic!("Invalid flags");
    }
}

#[derive(Debug)]
pub struct SftpFile {
    path: PathBuf,
    owner: Owner,
    flags: Flags,
}

impl SftpFile {
    pub fn new_server(path: PathBuf, file: File, flags: OpenFlags) -> Self {
        Self {
            path,
            owner: Owner::Server(file),
            flags: flags.into(),
        }
    }
}

impl SftpFile {
    pub async fn len(&self) -> usize {
        match &self.owner {
            Owner::Client(file) => file.bytes.get_ref().len(),
            Owner::Server(file) => file.metadata().await.unwrap().len() as usize,
        }
    }

    pub fn get_server_file(&self) -> Option<&File> {
        match &self.owner {
            Owner::Client(_) => None,
            Owner::Server(file) => Some(file),
        }
    }
}

impl AsyncWrite for SftpFile {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        if matches!(self.flags, Flags::ReadOnly) {
            return Poll::Ready(Err(Error::new(ErrorKind::Other, "File is read only")));
        }
        match &mut self.get_mut().owner {
            Owner::Client(file) => Pin::new(file).poll_write(cx, buf),
            Owner::Server(file) => Pin::new(file).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match &mut self.get_mut().owner {
            Owner::Client(file) => Pin::new(file).poll_flush(cx),
            Owner::Server(file) => Pin::new(file).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match &mut self.get_mut().owner {
            Owner::Client(file) => Pin::new(file).poll_shutdown(cx),
            Owner::Server(file) => Pin::new(file).poll_shutdown(cx),
        }
    }
}

impl AsyncRead for SftpFile {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        if matches!(self.flags, Flags::WriteOnly | Flags::AppendOnly) {
            return Poll::Ready(Err(Error::new(ErrorKind::Other, "File is write only")));
        }
        match &mut self.get_mut().owner {
            Owner::Client(file) => Pin::new(file).poll_read(cx, buf),
            Owner::Server(file) => Pin::new(file).poll_read(cx, buf),
        }
    }
}

impl AsyncSeek for SftpFile {
    fn start_seek(self: Pin<&mut Self>, position: SeekFrom) -> Result<(), Error> {
        match &mut self.get_mut().owner {
            Owner::Client(file) => Pin::new(file).start_seek(position),
            Owner::Server(file) => Pin::new(file).start_seek(position),
        }
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<u64, Error>> {
        match &mut self.get_mut().owner {
            Owner::Client(file) => Pin::new(file).poll_complete(cx),
            Owner::Server(file) => Pin::new(file).poll_complete(cx),
        }
    }
}


