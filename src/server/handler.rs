
use crate::protocol::{
    types::*,
    Status, StatusCode, VERSION,
};

/// Server handler for each client. This is `async_trait`
#[async_trait]
pub trait Handler: Sized {
    /// The type must have an Into<StatusCode>
    /// implementation because a response must be sent
    /// to any request, even if completed by error.
    type Error: Into<StatusCode>;

    /// Called by the handler when the packet is not implemented
    fn unimplemented(&self) -> Self::Error;

    /// The default is to send an SSH_FXP_VERSION response with
    /// the protocol version and ignore any extensions.
    #[allow(unused_variables)]
    async fn init(&mut self, arg: Init) -> Result<Version, Self::Error> {
        if arg.version != VERSION {
            panic!("version mismatch: {} != {}", arg.version, VERSION);
        }
        Ok(Version::default())
    }

    /// Called on SSH_FXP_OPEN
    #[allow(unused_variables)]
    async fn open(&mut self, arg: Open) -> Result<Handle, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_CLOSE.
    /// The status can be returned as Ok or as Err
    #[allow(unused_variables)]
    async fn close(&mut self, arg: Close) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_READ
    #[allow(unused_variables)]
    async fn read(&mut self, arg: Read) -> Result<Data, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_WRITE
    #[allow(unused_variables)]
    async fn write(&mut self, arg: Write) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_LSTAT
    #[allow(unused_variables)]
    async fn lstat(&mut self, arg: LStat) -> Result<Attrs, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_FSTAT
    #[allow(unused_variables)]
    async fn fstat(&mut self, arg: FStat) -> Result<Attrs, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_SETSTAT
    #[allow(unused_variables)]
    async fn setstat(&mut self, arg: SetStat) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_FSETSTAT
    #[allow(unused_variables)]
    async fn fsetstat(&mut self, arg: FSetStat) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_OPENDIR
    #[allow(unused_variables)]
    async fn opendir(&mut self, arg: OpenDir) -> Result<Handle, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_READDIR.
    /// EOF error should be returned at the end of reading the directory
    #[allow(unused_variables)]
    async fn readdir(&mut self, arg: ReadDir) -> Result<Name, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_REMOVE.
    /// The status can be returned as Ok or as Err
    #[allow(unused_variables)]
    async fn remove(&mut self, arg: Remove) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_MKDIR
    #[allow(unused_variables)]
    async fn mkdir(&mut self, arg: MkDir) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_RMDIR.
    /// The status can be returned as Ok or as Err
    #[allow(unused_variables)]
    async fn rmdir(&mut self, arg: RmDir) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_REALPATH.
    /// Must contain only one name and a dummy attributes
    #[allow(unused_variables)]
    async fn realpath(&mut self, arg: RealPath) -> Result<Name, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_STAT
    #[allow(unused_variables)]
    async fn stat(&mut self, arg: Stat) -> Result<Attrs, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_RENAME.
    /// The status can be returned as Ok or as Err
    #[allow(unused_variables)]
    async fn rename(&mut self, arg: Rename) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_READLINK
    #[allow(unused_variables)]
    async fn readlink(&mut self, arg: ReadLink) -> Result<Name, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_SYMLINK.
    /// The status can be returned as Ok or as Err
    #[allow(unused_variables)]
    async fn symlink(&mut self, arg: Symlink) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_EXTENDED.
    /// If the server does not recognize the `request' name
    /// the server must respond with an SSH_FX_OP_UNSUPPORTED error
    #[allow(unused_variables)]
    async fn extended(&mut self, arg: Extended) -> Result<ExtendedReply, Self::Error> {
        Err(self.unimplemented())
    }
}

