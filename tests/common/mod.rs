use std::path::Path;

#[allow(dead_code)]
pub async fn write(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> Result<(), async_fs_io::FsError> {
    async_fs_io::write_bytes(path, contents.as_ref()).await
}

#[allow(dead_code)]
pub async fn read(path: impl AsRef<Path>) -> Result<String, async_fs_io::FsError> {
    async_fs_io::read_string_bounded(path, 16 * 1024 * 1024).await
}

#[allow(dead_code)]
pub async fn ensure_dir(path: impl AsRef<Path>) -> Result<(), async_fs_io::FsError> {
    async_fs_io::ensure_dir(path).await
}

#[allow(dead_code)]
pub async fn metadata(path: impl AsRef<Path>) -> Result<std::fs::Metadata, async_fs_io::FsError> {
    async_fs_io::symlink_metadata(path).await
}
