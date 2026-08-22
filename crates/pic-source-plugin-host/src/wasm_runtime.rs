use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use profile_pulse_pic_source_plugin_api::{
    ContactContext, PicSourceCapability, PicSourceHostApi, PicSourceHostContext,
    PicSourcePluginError, PicSourcePluginMetadata, ProfilePicBytes, ProfilePicCandidate,
    ProfilePicSourcePlugin,
};
use wasmtime::{Caller, Engine, Instance, Linker, Module, Store};

use crate::desktop_host::{guess_content_type, DesktopHostApi};
use crate::error::HostError;

const GUEST_HEAP: i32 = 256 * 1024;

#[derive(Clone)]
struct WasmHostState {
    host: Arc<DesktopHostApi>,
    ctx: PicSourceHostContext,
    approved: Vec<PicSourceCapability>,
}

pub struct WasmPicSourcePlugin {
    metadata: PicSourcePluginMetadata,
    capabilities: Vec<PicSourceCapability>,
    approved: Vec<PicSourceCapability>,
    wasm_path: PathBuf,
    host: Arc<DesktopHostApi>,
    engine: Engine,
    module: Module,
}

impl WasmPicSourcePlugin {
    pub fn from_install_dir(
        install_dir: &Path,
        host: Arc<DesktopHostApi>,
        approved: Vec<PicSourceCapability>,
    ) -> Result<Self, HostError> {
        let manifest_bytes = std::fs::read(install_dir.join(crate::manifest::MANIFEST_FILE))?;
        let manifest = crate::manifest::PicSourcePluginManifest::parse(&manifest_bytes)?;
        manifest.validate()?;
        let metadata = manifest.to_metadata()?;
        let capabilities = manifest.requested_capabilities();
        let wasm_path = install_dir.join(crate::manifest::WASM_FILE);
        if !wasm_path.exists() {
            return Err(HostError::InvalidPackage(format!(
                "missing {}",
                crate::manifest::WASM_FILE
            )));
        }

        let engine = Engine::default();
        let module = Module::from_file(&engine, &wasm_path)
            .map_err(|e| HostError::Wasm(e.to_string()))?;

        Ok(Self {
            metadata,
            capabilities,
            approved,
            wasm_path,
            host,
            engine,
            module,
        })
    }

    fn call_json<T: serde::de::DeserializeOwned>(
        &self,
        export: &str,
        input: &[u8],
    ) -> Result<T, PicSourcePluginError> {
        let state = WasmHostState {
            host: Arc::clone(&self.host),
            ctx: PicSourceHostContext {
                plugin_id: self.metadata.id.clone(),
            },
            approved: self.approved.clone(),
        };
        let mut store = Store::new(&self.engine, state);
        let mut linker = Linker::new(&self.engine);
        link_host_functions(&mut linker)?;

        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;

        reset_guest(&mut store, &instance)?;
        let output = invoke_guest_json(&mut store, &instance, export, input)?;
        serde_json::from_slice(&output)
            .map_err(|e| PicSourcePluginError::Internal(format!("invalid plugin JSON: {e}")))
    }
}

fn link_host_functions(linker: &mut Linker<WasmHostState>) -> Result<(), PicSourcePluginError> {
    linker
        .func_wrap(
            "env",
            "http_get",
            |mut caller: Caller<'_, WasmHostState>,
             url_ptr: i32,
             url_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> i32 {
                if !caller.data().approved.contains(&PicSourceCapability::Network) {
                    return -403;
                }
                let url = match read_guest_string(&mut caller, url_ptr, url_len) {
                    Ok(v) => v,
                    Err(code) => return code,
                };
                let host = Arc::clone(&caller.data().host);
                let ctx = caller.data().ctx.clone();
                let result = tokio::runtime::Runtime::new()
                    .expect("tokio runtime")
                    .block_on(async { host.http_get(&ctx, &url, &[]).await });
                match result {
                    Ok(bytes) => write_guest_bytes(&mut caller, out_ptr, out_cap, &bytes),
                    Err(PicSourcePluginError::NotFound) => -404,
                    Err(_) => -500,
                }
            },
        )
        .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;
    Ok(())
}

fn reset_guest(
    store: &mut Store<WasmHostState>,
    instance: &Instance,
) -> Result<(), PicSourcePluginError> {
    if let Ok(reset) = instance.get_typed_func::<(), ()>(&mut *store, "plugin_reset") {
        reset
            .call(&mut *store, ())
            .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;
    }
    Ok(())
}

fn invoke_guest_json(
    store: &mut Store<WasmHostState>,
    instance: &Instance,
    export: &str,
    input: &[u8],
) -> Result<Vec<u8>, PicSourcePluginError> {
    let alloc = instance
        .get_typed_func::<i32, i32>(&mut *store, "plugin_alloc")
        .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;
    let invoke = instance
        .get_typed_func::<(i32, i32, i32, i32), i32>(&mut *store, export)
        .map_err(|e| PicSourcePluginError::Internal(format!("missing export `{export}`: {e}")))?;

    let in_ptr = alloc
        .call(&mut *store, input.len() as i32)
        .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;
    if in_ptr < 0 {
        return Err(PicSourcePluginError::Internal("plugin_alloc failed".into()));
    }
    write_guest_bytes_raw(store, instance, in_ptr, input)?;

    let out_ptr = alloc
        .call(&mut *store, GUEST_HEAP)
        .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;
    if out_ptr < 0 {
        return Err(PicSourcePluginError::Internal("plugin_alloc failed".into()));
    }

    let written = invoke
        .call(
            &mut *store,
            (in_ptr, input.len() as i32, out_ptr, GUEST_HEAP),
        )
        .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;
    if written < 0 {
        return Err(PicSourcePluginError::Internal(format!(
            "plugin `{export}` returned error code {written}"
        )));
    }
    read_guest_bytes(store, instance, out_ptr, written as usize)
}

fn guest_memory(
    store: &mut Store<WasmHostState>,
    instance: &Instance,
) -> Result<wasmtime::Memory, PicSourcePluginError> {
    instance
        .get_memory(store, "memory")
        .ok_or_else(|| PicSourcePluginError::Internal("plugin missing memory export".into()))
}

fn read_guest_string(
    caller: &mut Caller<'_, WasmHostState>,
    ptr: i32,
    len: i32,
) -> Result<String, i32> {
    if ptr < 0 || len < 0 {
        return Err(-1);
    }
    let memory = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or(-1)?;
    let mut buf = vec![0u8; len as usize];
    memory.read(caller, ptr as usize, &mut buf).map_err(|_| -1)?;
    String::from_utf8(buf).map_err(|_| -1)
}

fn write_guest_bytes(
    caller: &mut Caller<'_, WasmHostState>,
    out_ptr: i32,
    out_cap: i32,
    bytes: &[u8],
) -> i32 {
    if out_ptr < 0 || out_cap < 0 || bytes.len() > out_cap as usize {
        return -1;
    }
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return -1,
    };
    if memory.write(caller, out_ptr as usize, bytes).is_err() {
        return -1;
    }
    bytes.len() as i32
}

fn write_guest_bytes_raw(
    store: &mut Store<WasmHostState>,
    instance: &Instance,
    ptr: i32,
    bytes: &[u8],
) -> Result<(), PicSourcePluginError> {
    let memory = guest_memory(store, instance)?;
    memory
        .write(store, ptr as usize, bytes)
        .map_err(|e| PicSourcePluginError::Internal(e.to_string()))
}

fn read_guest_bytes(
    store: &mut Store<WasmHostState>,
    instance: &Instance,
    ptr: i32,
    len: usize,
) -> Result<Vec<u8>, PicSourcePluginError> {
    let memory = guest_memory(store, instance)?;
    let mut buf = vec![0u8; len];
    memory
        .read(store, ptr as usize, &mut buf)
        .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;
    Ok(buf)
}

#[async_trait]
impl ProfilePicSourcePlugin for WasmPicSourcePlugin {
    fn metadata(&self) -> PicSourcePluginMetadata {
        self.metadata.clone()
    }

    fn capabilities(&self) -> Vec<PicSourceCapability> {
        self.capabilities.clone()
    }

    async fn discover_sources(
        &self,
        ctx: &ContactContext,
    ) -> Result<Vec<ProfilePicCandidate>, PicSourcePluginError> {
        let input = serde_json::to_vec(ctx)
            .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;
        self.call_json("discover", &input)
    }

    async fn fetch_pic(
        &self,
        candidate: &ProfilePicCandidate,
    ) -> Result<ProfilePicBytes, PicSourcePluginError> {
        let input = serde_json::to_vec(&candidate.fetch_token)
            .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;
        let mut pic: ProfilePicBytes = self.call_json("fetch_pic", &input)?;
        if pic.content_type.is_empty() {
            pic.content_type = guess_content_type(&pic.bytes);
        }
        Ok(pic)
    }
}

impl std::fmt::Debug for WasmPicSourcePlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmPicSourcePlugin")
            .field("id", &self.metadata.id)
            .field("wasm_path", &self.wasm_path)
            .finish()
    }
}
