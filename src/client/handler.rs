use crate::protocol::{
    types::*,
    VERSION, StatusCode, Status
};

/// Client handler for each client. This is [`async_trait::async_trait`]
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
    async fn version(&mut self, arg: Version) -> Result<(), Self::Error> {
        if arg.version != VERSION {
            panic!("version mismatch: {} != {}", arg.version, VERSION);
        }
        Ok(())
    }

    /// Called on SSH_FXP_HANDLE
    #[allow(unused_variables)]
    async fn handle(&mut self, id: u32, handle: String) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_DATA
    #[allow(unused_variables)]
    async fn data(&mut self, arg: Data) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_NAME
    #[allow(unused_variables)]
    async fn name(&mut self, arg: Name) -> Result<Name, Self::Error> {
        Err(self.unimplemented())
    }

    /// Called on SSH_FXP_ATTRS
    #[allow(unused_variables)]
    async fn attrs(&mut self, arg: Attrs) -> Result<Status, Self::Error> {
        Err(self.unimplemented())
    }
}

