use chrono::{DateTime, Utc};
use serde::{Serialize, ser::SerializeStruct};
use std::time::{Duration, UNIX_EPOCH};

use super::{impl_packet_for, impl_request_id, Packet, RequestId, FileAttributes};

/// Implementation for SSH_FXP_NAME
#[derive(Debug, Serialize, Deserialize)]
pub struct Name {
    pub id: u32,
    pub files: Vec<File>,
}

impl_request_id!(Name);
impl_packet_for!(Name);

#[derive(Debug, Deserialize)]
pub struct File {
    pub filename: String,
    pub longname: String,
    pub attrs: FileAttributes,
}

impl File {
    fn permission(&self, permission: u32) -> String {
        let read = (permission >> 2) & 0x1;
        let write = (permission >> 1) & 0x1;
        let execute = permission & 0x1;

        let read = if read == 0x1 { "r" } else { "-" };
        let write = if write == 0x01 { "w" } else { "-" };
        let execute = if execute == 0x01 { "x" } else { "-" };

        format!("{}{}{}", read, write, execute)
    }

    fn permissions(&self) -> String {
        let permissions = self.attrs.permissions.unwrap_or(0);

        let owner = self.permission((permissions >> 6) & 0x7);
        let group = self.permission((permissions >> 3) & 0x7);
        let other = self.permission(permissions & 0x7);

        let directory = if self.attrs.is_dir() { "d" } else { "-" };

        format!("{}{}{}{}", directory, owner, group, other)
    }

    /// Get formed longname
    pub fn longname(&self) -> String {
        let permissions = self.permissions();
        let size = self.attrs.size.unwrap_or(0);
        let uid = self.attrs.uid.unwrap_or(0);
        let gid = self.attrs.gid.unwrap_or(0);
        let mtime = self.attrs.mtime.unwrap_or(0);

        let date = UNIX_EPOCH + Duration::from_secs(mtime as u64);
        let datetime = DateTime::<Utc>::from(date);
        let delayed = datetime.format("%b %d %Y %H:%M");

        format!(
            "{} 0 {} {} {} {} {}",
            permissions,
            if let Some(user) = &self.attrs.user {
                user.to_string()
            } else {
                uid.to_string()
            },
            if let Some(group) = &self.attrs.group {
                group.to_string()
            } else {
                gid.to_string()
            },
            size,
            delayed,
            self.filename
        )
    }
}

impl Serialize for File {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("File", 3)?;
        s.serialize_field("filename", &self.filename)?;
        s.serialize_field("longname", &self.longname())?;
        s.serialize_field("attrs", &self.attrs)?;
        s.end()
    }
}