use std::{collections::HashMap, io::SeekFrom, path::PathBuf};

use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::{
    protocol::{types::*, Status, StatusCode, VERSION},
    sftp_fs::file::SftpFile,
};

#[derive(Debug, Default)]
pub struct SftpServerHandleImpl {
    files: HashMap<String, SftpFile>,
    dir: HashMap<String, PathBuf>
}

#[async_trait::async_trait]
impl super::Handler for SftpServerHandleImpl {
    type Error = StatusCode;

    fn unimplemented(&self) -> Self::Error {
        StatusCode::OpUnsupported
    }

    async fn init(&mut self, arg: Init) -> Result<Version, Self::Error> {
        if arg.version != VERSION {
            return Err(self.unimplemented());
        }
        Ok(Version::default())
    }

    async fn open(&mut self, arg: Open) -> Result<Handle, Self::Error> {
        let path = PathBuf::from(arg.filename);
        let flags = arg.pflags;

        //exclude and truncate are only valid with create
        if (flags.exclude() || flags.truncate()) && !flags.create() {
            return Err(self.unimplemented());
        }

        //if create is false, the file must exist
        if !flags.create() && !path.exists() {
            return Err(self.unimplemented());
        }

        //if exclusive is true, the file must not exist
        if flags.exclude() && path.exists() {
            return Err(self.unimplemented());
        }

        let file = tokio::fs::OpenOptions::new()
            .read(flags.read())
            .write(flags.write())
            .create(flags.create())
            .append(flags.append())
            .truncate(flags.truncate())
            .open(&path)
            .await
            .map_err(|_| StatusCode::Failure)?;

        //limit path in handle str to 245 chars
        let handle_str = format!("f:{}{:?}", arg.id, path)[..245].to_string();

        self.files
            .insert(handle_str.clone(), SftpFile::new_server(path, file, flags));

        Ok(Handle {
            id: arg.id,
            handle: handle_str,
        })
    }

    async fn close(&mut self, arg: Close) -> Result<Status, Self::Error> {
        let file_handle = arg.handle;
        if let Some(mut file) = self.files.remove(&file_handle) {
            file.shutdown().await.map_err(|_| StatusCode::Failure)?;
            drop(file);
            Ok(Status {
                id: arg.id,
                error_message: String::new(),
                status_code: StatusCode::Ok,
                language_tag: "en-US".to_string(),
            })
        } else {
            Err(StatusCode::NoSuchFile)
        }
    }

    async fn read(&mut self, arg: Read) -> Result<Data, Self::Error> {
        let file_handle = arg.handle;
        if let Some(file) = self.files.get_mut(&file_handle) {
            let starting_offset = arg.offset as usize;
            let file_len = file.len().await;
            if starting_offset > file_len {
                return Err(StatusCode::Eof);
            }
            let buffer_len = std::cmp::min(arg.len as usize, file_len - starting_offset);

            let mut buffer = vec![0; buffer_len];
            file.seek(SeekFrom::Start(starting_offset as u64))
                .await
                .map_err(|_| StatusCode::Failure)?;

            file.read_exact(&mut buffer)
                .await
                .map_err(|_| StatusCode::Failure)?;

            Ok(Data {
                id: arg.id,
                data: buffer,
            })
        } else {
            Err(StatusCode::NoSuchFile)
        }
    }

    async fn write(&mut self, arg: Write) -> Result<Status, Self::Error> {
        let file_handle = arg.handle;
        if let Some(file) = self.files.get_mut(&file_handle) {
            let starting_offset = arg.offset as usize;
            let file_len = file.len().await;
            if starting_offset > file_len {
                return Err(StatusCode::Eof);
            }
            let buffer_len = std::cmp::min(arg.data.len(), file_len - starting_offset);

            file.seek(SeekFrom::Start(starting_offset as u64))
                .await
                .map_err(|_| StatusCode::Failure)?;

            file.write_all(&arg.data[..buffer_len])
                .await
                .map_err(|_| StatusCode::Failure)?;

            Ok(Status {
                id: arg.id,
                error_message: String::new(),
                status_code: StatusCode::Ok,
                language_tag: "en-US".to_string(),
            })
        } else {
            Err(StatusCode::NoSuchFile)
        }
    }

    async fn lstat(&mut self, arg: LStat) -> Result<Attrs, Self::Error> {
        //get the file attributes without following symlinks
        let path = PathBuf::from(arg.path);
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|_| StatusCode::NoSuchFile)?;
        Ok(Attrs {
            id: arg.id,
            attrs: (&metadata).into(),
        })
    }

    async fn fstat(&mut self, arg: FStat) -> Result<Attrs, Self::Error> {
        let file_handle = arg.handle;
        if let Some(file) = self.files.get_mut(&file_handle) {
            let metadata = file
                .get_server_file()
                .unwrap()
                .metadata()
                .await
                .map_err(|_| StatusCode::Failure)?;
            Ok(Attrs {
                id: arg.id,
                attrs: (&metadata).into(),
            })
        } else {
            Err(StatusCode::NoSuchFile)
        }
    }

    async fn setstat(&mut self, arg: SetStat) -> Result<Status, Self::Error> {
        let file_attr = arg.attrs;
        let path = PathBuf::from(arg.path);
        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|_| StatusCode::NoSuchFile)?;
        let mut permissions = metadata.permissions();
        #[cfg(windows)]
        {
            if let Some(desired_perm) = file_attr.permissions {
                permissions.set_readonly(desired_perm & 0o555 == 0);
            }
        }
        #[cfg(unix)]
        {
            if let Some(desired_perm) = file_attr.permissions {
                permissions.set_mode(desired_perm);
            }
        }
        tokio::fs::set_permissions(&path, permissions)
            .await
            .map_err(|_| StatusCode::Failure)?;
        Ok(Status {
            id: arg.id,
            error_message: String::new(),
            status_code: StatusCode::Ok,
            language_tag: "en-US".to_string(),
        })
    }

    async fn fsetstat(&mut self, arg: FSetStat) -> Result<Status, Self::Error> {
        let file_attr = arg.attrs;
        let file_handle = arg.handle;
        if let Some(file) = self.files.get_mut(&file_handle) {
            let mut permissions = file
                .get_server_file()
                .unwrap()
                .metadata()
                .await
                .map_err(|_| StatusCode::Failure)?
                .permissions();
            #[cfg(windows)]
            {
                if let Some(desired_perm) = file_attr.permissions {
                    permissions.set_readonly(desired_perm & 0o555 == 0);
                }
            }
            #[cfg(unix)]
            {
                if let Some(desired_perm) = file_attr.permissions {
                    permissions.set_mode(desired_perm);
                }
            }
            file.get_server_file()
                .unwrap()
                .set_permissions(permissions)
                .await
                .map_err(|_| StatusCode::Failure)?;
            Ok(Status {
                id: arg.id,
                error_message: String::new(),
                status_code: StatusCode::Ok,
                language_tag: "en-US".to_string(),
            })
        } else {
            Err(StatusCode::NoSuchFile)
        }
    }

    async fn opendir(&mut self, arg: OpenDir) -> Result<Handle, Self::Error> {
        let path = PathBuf::from(arg.path);
        let handle_str = format!("d:{}{:?}", arg.id, path)[..245].to_string();
        if path.is_dir() {
            self.dir.insert(handle_str.clone(), path);
            Ok(Handle {
                id: arg.id,
                handle: handle_str,
            })
        } else {
            Err(StatusCode::NoSuchFile)
        }
    }

    async fn readdir(&mut self, arg: ReadDir) -> Result<Name, Self::Error> {
        let dir_handle = arg.handle;
        if let Some(path) = self.dir.get(&dir_handle) {
            let mut files = Vec::new();

            let mut dir_reader = tokio::fs::read_dir(path)
                .await
                .map_err(|_| StatusCode::Failure)?;

            while let Ok(Some(entry)) = dir_reader.next_entry().await {
                let metadata = entry.metadata().await.map_err(|_| StatusCode::Failure)?;
                let filename = entry.file_name().into_string().unwrap();
                files.push(File {
                    filename,
                    longname: String::new(),
                    attrs: (&metadata).into(),
                });
            }
            Ok(Name {
                id: arg.id,
                files,
            })
        } else {
            Err(StatusCode::NoSuchFile)
        }
    }

    async fn remove(&mut self, arg: Remove) -> Result<Status, Self::Error> {
        let path = PathBuf::from(arg.filename);
        tokio::fs::remove_file(&path)
            .await
            .map_err(|_| StatusCode::NoSuchFile)?;
        Ok(Status {
            id: arg.id,
            error_message: String::new(),
            status_code: StatusCode::Ok,
            language_tag: "en-US".to_string(),
        })
    }

    async fn mkdir(&mut self, arg: MkDir) -> Result<Status, Self::Error> {
        let path = PathBuf::from(arg.path);
        //TODO: handle attrs
        // let attrs = arg.attrs;
        tokio::fs::DirBuilder::new()
            .recursive(true)
            .create(&path)
            .await
            .map_err(|_| StatusCode::Failure)?;
        Ok(Status {
            id: arg.id,
            error_message: String::new(),
            status_code: StatusCode::Ok,
            language_tag: "en-US".to_string(),
        })
    }

    async fn rmdir(&mut self, arg: RmDir) -> Result<Status, Self::Error> {
        let path = PathBuf::from(arg.path);
        tokio::fs::remove_dir(&path)
            .await
            .map_err(|_| StatusCode::NoSuchFile)?;
        Ok(Status {
            id: arg.id,
            error_message: String::new(),
            status_code: StatusCode::Ok,
            language_tag: "en-US".to_string(),
        })
    }

    async fn realpath(&mut self, arg: RealPath) -> Result<Name, Self::Error> {
        let path = PathBuf::from(arg.path);
        let real_path = tokio::fs::canonicalize(&path)
            .await
            .map_err(|_| StatusCode::NoSuchFile)?;
        Ok(Name {
            id: arg.id,
            files: vec![File {
                filename: real_path.to_str().unwrap().to_string(),
                longname: String::new(),
                attrs: FileAttributes::default(),
            }]
        })
    }

    async fn rename(&mut self, arg: Rename) -> Result<Status, Self::Error> {
        let old_path = PathBuf::from(arg.oldpath);
        let new_path = PathBuf::from(arg.newpath);
        tokio::fs::rename(&old_path, &new_path)
            .await
            .map_err(|_| StatusCode::NoSuchFile)?;
        Ok(Status {
            id: arg.id,
            error_message: String::new(),
            status_code: StatusCode::Ok,
            language_tag: "en-US".to_string(),
        })
    }

    async fn readlink(&mut self, arg: ReadLink) -> Result<Name, Self::Error> {
        let path = PathBuf::from(arg.path);
        let link_path = tokio::fs::read_link(&path)
            .await
            .map_err(|_| StatusCode::NoSuchFile)?;
        Ok(Name {
            id: arg.id,
            files: vec![File {
                filename: link_path.to_str().unwrap().to_string(),
                longname: String::new(),
                attrs: FileAttributes::default(),
            }]
        })
    }

    async fn symlink(&mut self, arg: Symlink) -> Result<Status, Self::Error> {
        let link_path = PathBuf::from(arg.linkpath);
        let target_path = PathBuf::from(arg.targetpath);
        #[cfg(windows)]
        {
            //if target path is a directory then use symlink_dir
            if target_path.is_dir() {
                tokio::fs::symlink_dir(&target_path, &link_path)
                    .await
                    .map_err(|_| StatusCode::Failure)?;
            } else {
                tokio::fs::symlink_file(&target_path, &link_path)
                    .await
                    .map_err(|_| StatusCode::Failure)?;
            }
        }
        #[cfg(unix)]
        {
            tokio::fs::symlink(&target_path, &link_path)
                .await
                .map_err(|_| StatusCode::Failure)?;
        }
        Ok(Status {
            id: arg.id,
            error_message: String::new(),
            status_code: StatusCode::Ok,
            language_tag: "en-US".to_string(),
        })
    }
}
