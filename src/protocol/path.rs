use super::{impl_request_id, RequestId};

/// Implementation for SSH_FXP_... LSTAT, OPENDIR,
/// RMDIR, REALPATH, STAT and READLINK
#[derive(Debug, Serialize, Deserialize)]
pub struct Path {
    pub id: u32,
    pub path: String,
}

pub type LStat = Path;
pub type OpenDir = Path;
pub type RmDir = Path;
pub type RealPath = Path;
pub type Stat = Path;
pub type ReadLink = Path;

impl_request_id!(Path);
