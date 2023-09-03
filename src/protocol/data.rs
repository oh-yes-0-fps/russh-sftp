use super::{impl_packet_for, impl_request_id, Packet, RequestId};

/// Implementation for [SSH_FXP_DATA](crate::protocol::SSH_FXP_DATA)
/// The SSH_FXP_DATA response has the following format: <br>
/// id: u32 <br>
/// data: Vec<u8> <br>
/// Where [`Data::id`] is the request identifier, and [`Data::data`] is an arbitrary byte
/// string containing the requested data.  The data string may be at most
/// the number of bytes requested in a [SSH_FXP_READ](crate::protocol::SSH_FXP_READ) request,
/// but may also be shorter if end of file is reached or if the read is from something
/// other than a regular file.
#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub id: u32,
    pub data: Vec<u8>,
}

impl_request_id!(Data);
impl_packet_for!(Data);
