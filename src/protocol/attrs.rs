use super::{impl_packet_for, impl_request_id, Packet, RequestId, FileAttributes};

/// Implementation for SSH_FXP_ATTRS
/// Where [`Attrs::id`] is the request identifier, and [`Attrs::attrs`] is the returned
/// file attributes as described in Section [`FileAttributes`].
#[derive(Debug, Serialize, Deserialize)]
pub struct Attrs {
    pub id: u32,
    pub attrs: FileAttributes,
}

impl_request_id!(Attrs);
impl_packet_for!(Attrs);
