use super::{impl_request_id, FileAttributes, RequestId};

/// Implementation for [SSH_FXP_FSETSTAT](crate::protocol::SSH_FXP_FSETSTAT)
#[derive(Debug, Serialize, Deserialize)]
pub struct FSetStat {
    pub id: u32,
    pub handle: String,
    pub attrs: FileAttributes,
}

impl_request_id!(FSetStat);
