//! One upload attempt end to end: source, resumable stream, progress, entry.

use std::sync::Arc;

use cloudreve_sdk_api::Client;
use cloudreve_sdk_api::api::ExplorerApi;

use crate::remote::UploadUriError;

use super::*;

impl CloudreveStorage {
    pub(crate) async fn upload_asset(
        &self,
        request: UploadRequest,
    ) -> Result<RemoteEntry, UploadUriError> {
        let source = build_upload_source(&request.source_uri, request.size)?;
        let client = self.client().map_err(UploadUriError::from_remote)?;
        let remote_uri = join_uri(&request.dest_uri, &request.name);
        let uploader = Uploader::new(Arc::clone(&client), UploaderConfig::default());
        let params = UploadParams {
            source: source.clone(),
            remote_uri: remote_uri.clone(),
            mime_type: request.mime.clone(),
            last_modified: None,
            overwrite: false,
            policy_id: String::new(),
            metadata: None,
            previous_version: String::new(),
        };

        // The stored session is what lets an interrupted upload pick up the
        // chunks the server already holds. It is keyed by the queue task, so a
        // retry of the same entry resumes while a second upload of the same file
        // does not share or clobber its chunks.
        let key = request.task_id.0.clone();

        let (stream, records) =
            open_upload(&uploader, self.sessions.as_ref(), &key, params).await?;

        drive_upload(
            stream,
            records,
            self.sessions.as_ref(),
            key.as_str(),
            &request.cancel,
            &request.progress,
        )
        .await?;

        // Cancellation between `Completed` and the read-back would write the
        // asset locally for a file the user gave up on; the session is already
        // gone at that point, so this only skips the local bookkeeping.
        if request.cancel.is_cancelled() {
            return Err(UploadUriError::Interrupted);
        }

        read_back_entry(&client, &remote_uri).await
    }
}

/// Metadata extraction may lag behind the upload, so retry briefly until the
/// new asset can be read back with its id and timestamps.
async fn read_back_entry(client: &Client, remote_uri: &str) -> Result<RemoteEntry, UploadUriError> {
    for attempt in 0..5 {
        match client
            .get_file_info(&GetFileInfoService {
                uri: Some(remote_uri.to_string()),
                id: None,
                extended: Some(false),
                folder_summary: Some(false),
            })
            .await
        {
            Ok(info) => return Ok(to_entry(remote_uri, &info)),
            Err(_) if attempt < 4 => {
                tokio::time::sleep(std::time::Duration::from_millis(250 * (attempt + 1))).await;
            }
            Err(err) => return Err(UploadUriError::from_api(err)),
        }
    }
    Err(UploadUriError::ChunkFailed {
        cause: "上传完成但无法读取文件信息".to_string(),
    })
}
