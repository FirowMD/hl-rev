use crate::app_data::Storage;
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use tauri::{AppHandle, Manager};

pub(crate) fn serialize_loaded_bytecode(app_handle: &AppHandle) -> McpResult<Vec<u8>> {
    let state = app_handle.state::<Storage>();
    let app_data = state
        .bytecode
        .lock()
        .map_err(|e| McpError::Internal(e.to_string()))?;
    let bytecode = app_data
        .bytecode
        .as_ref()
        .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

    let mut candidate = bytecode.clone();
    candidate
        .rebuild_derived_data()
        .map_err(|e| McpError::Validation(e.to_string()))?;
    candidate
        .validate()
        .map_err(|e| McpError::Validation(e.to_string()))?;

    let mut serialized = Vec::new();
    candidate
        .serialize(&mut serialized)
        .map_err(|e| McpError::Internal(e.to_string()))?;
    Ok(serialized)
}

pub(crate) fn atomic_write(
    file_path: &Path,
    contents: &[u8],
    allow_overwrite: bool,
) -> McpResult<()> {
    let directory = file_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty());
    let directory = directory.unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(directory)
        .map_err(|e| McpError::Internal(e.to_string()))?;
    temporary
        .write_all(contents)
        .and_then(|_| temporary.flush())
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|e| McpError::Internal(e.to_string()))?;

    let persisted = if allow_overwrite {
        temporary.persist(file_path)
    } else {
        temporary.persist_noclobber(file_path)
    }
    .map_err(|e| McpError::Internal(e.error.to_string()))?;
    persisted
        .sync_all()
        .map_err(|e| McpError::Internal(e.to_string()))?;
    Ok(())
}

#[derive(Clone)]
pub struct SaveBytecodeHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for SaveBytecodeHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'file_path'".to_string()))?;

        let serialized = serialize_loaded_bytecode(&self.app_handle)?;
        atomic_write(Path::new(file_path), &serialized, false)?;

        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "save_bytecode".to_string(),
            Some(
                "Validate and atomically save bytecode to a new file without discarding debug data"
                    .to_string(),
            ),
            json!({
                "type": "object",
                "properties": {"file_path": {"type": "string"}},
                "required": ["file_path"],
                "additionalProperties": false
            }),
            SaveBytecodeHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_requires_explicit_overwrite() {
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("output.hl");
        std::fs::write(&destination, b"original").unwrap();

        assert!(atomic_write(&destination, b"rejected", false).is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), b"original");

        atomic_write(&destination, b"replacement", true).unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), b"replacement");
    }
}
