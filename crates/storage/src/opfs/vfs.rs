//! Origin Private File System helpers for the web PWA build.
use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetDirectoryOptions,
    FileSystemGetFileOptions, FileSystemWritableFileStream,
};

const ROOT_NAME: &str = "profile-pulse";

fn opfs_error(context: &str, err: wasm_bindgen::JsValue) -> String {
    format!("{context}: {err:?}")
}

async fn storage_root() -> Result<FileSystemDirectoryHandle, String> {
    let window = web_sys::window().ok_or("browser window unavailable")?;
    let storage = window
        .navigator()
        .storage()
        .map_err(|err| opfs_error("navigator.storage", err))?;
    let root = JsFuture::from(storage.get_directory())
        .await
        .map_err(|err| opfs_error("storage.get_directory", err))?;
    root.dyn_into()
        .map_err(|err| opfs_error("storage root cast", err))
}

async fn open_dir(relative_path: &str, create: bool) -> Result<FileSystemDirectoryHandle, String> {
    let mut current = storage_root().await?;
    for segment in relative_path.split('/').filter(|part| !part.is_empty()) {
        let options = FileSystemGetDirectoryOptions::new();
        options.set_create(create);
        let next = JsFuture::from(
            current
                .get_directory_handle_with_options(segment, &options)
                .map_err(|err| opfs_error("get_directory_handle", err))?,
        )
        .await
        .map_err(|err| opfs_error("get_directory_handle await", err))?;
        current = next
            .dyn_into()
            .map_err(|err| opfs_error("directory handle cast", err))?;
    }
    Ok(current)
}

async fn open_file(relative_path: &str, create: bool) -> Result<FileSystemFileHandle, String> {
    let path = relative_path.trim_start_matches('/');
    let (dir_path, file_name) = path
        .rsplit_once('/')
        .ok_or_else(|| format!("invalid opfs path: {relative_path}"))?;
    let dir = open_dir(&format!("{ROOT_NAME}/{dir_path}"), create).await?;
    let options = FileSystemGetFileOptions::new();
    options.set_create(create);
    let handle = JsFuture::from(
        dir.get_file_handle_with_options(file_name, &options)
            .map_err(|err| opfs_error("get_file_handle", err))?,
    )
    .await
    .map_err(|err| opfs_error("get_file_handle await", err))?;
    handle
        .dyn_into()
        .map_err(|err| opfs_error("file handle cast", err))
}

pub async fn exists(relative_path: &str) -> Result<bool, String> {
    match read_bytes(relative_path).await {
        Ok(_) => Ok(true),
        Err(err) if err.contains("not found") => Ok(false),
        Err(err) => Err(err),
    }
}

pub async fn read_bytes(relative_path: &str) -> Result<Vec<u8>, String> {
    let handle = open_file(relative_path, false).await?;
    let file = JsFuture::from(
        handle
            .get_file()
            .map_err(|err| opfs_error("get_file", err))?,
    )
    .await
    .map_err(|err| opfs_error("get_file await", err))?;
    let file: web_sys::File = file
        .dyn_into()
        .map_err(|err| opfs_error("file cast", err))?;
    let buffer = JsFuture::from(
        file.array_buffer()
            .map_err(|err| opfs_error("array_buffer", err))?,
    )
    .await
    .map_err(|err| opfs_error("array_buffer await", err))?;
    Ok(Uint8Array::new(&buffer).to_vec())
}

pub async fn read_string(relative_path: &str) -> Result<String, String> {
    let bytes = read_bytes(relative_path).await?;
    String::from_utf8(bytes).map_err(|err| format!("utf-8 decode failed: {err}"))
}

pub async fn write_bytes(relative_path: &str, bytes: &[u8]) -> Result<(), String> {
    let handle = open_file(relative_path, true).await?;
    let writable = JsFuture::from(
        handle
            .create_writable()
            .map_err(|err| opfs_error("create_writable", err))?,
    )
    .await
    .map_err(|err| opfs_error("create_writable await", err))?;
    let writable: FileSystemWritableFileStream = writable
        .dyn_into()
        .map_err(|err| opfs_error("writable stream cast", err))?;
    let array = Uint8Array::from(bytes);
    JsFuture::from(
        writable
            .write_with_buffer_source(&array)
            .map_err(|err| opfs_error("writable.write", err))?,
    )
    .await
    .map_err(|err| opfs_error("writable.write await", err))?;
    JsFuture::from(
        writable
            .close()
            .map_err(|err| opfs_error("writable.close", err))?,
    )
    .await
    .map_err(|err| opfs_error("writable.close await", err))?;
    Ok(())
}

pub async fn write_string(relative_path: &str, text: &str) -> Result<(), String> {
    write_bytes(relative_path, text.as_bytes()).await
}

pub async fn remove_file(relative_path: &str) -> Result<(), String> {
    let path = relative_path.trim_start_matches('/');
    let (dir_path, file_name) = path
        .rsplit_once('/')
        .ok_or_else(|| format!("invalid opfs path: {relative_path}"))?;
    let dir = open_dir(&format!("{ROOT_NAME}/{dir_path}"), false).await?;
    JsFuture::from(
        dir.remove_entry(file_name)
            .map_err(|err| opfs_error("remove_entry", err))?,
    )
    .await
    .map_err(|err| opfs_error("remove_entry await", err))?;
    Ok(())
}

pub async fn ensure_dir(relative_path: &str) -> Result<(), String> {
    let _ = open_dir(&format!("{ROOT_NAME}/{relative_path}"), true).await?;
    Ok(())
}
