use super::{impl_request_id, FileAttributes, RequestId};

/// Implementation for SSH_FXP_... SETSTAT and MKDIR
#[derive(Debug, Serialize, Deserialize)]
pub struct PathAttrs {
    pub id: u32,
    pub path: String,
    pub attrs: FileAttributes,
}

pub type SetStat = PathAttrs;
pub type MkDir = PathAttrs;

impl_request_id!(PathAttrs);
