use std::path::PathBuf;

use base64::{engine::general_purpose, Engine};
use mime_guess::from_path;

use crate::ai::ambient_agents::task::AttachmentInput;
use crate::ai::attachment_utils::MAX_ATTACHMENT_SIZE_BYTES;
use crate::util::image::MIN_IMAGE_HEADER_SIZE;

/// Maximum number of file attachments for a cloud agent task.
pub const MAX_ATTACHMENT_COUNT_FOR_CLOUD_QUERY: usize = 25;

/// Process a file attachment for ambient agent upload.
/// Returns AttachmentInput with base64-encoded data.
/// All file types share the same 10MB size limit.
pub fn process_attachment(
    attachment_path: &PathBuf,
    index: usize,
) -> anyhow::Result<AttachmentInput> {
    let file_bytes = std::fs::read(attachment_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read attachment file '{}': {e}",
            attachment_path.display()
        )
    })?;

    // Detect MIME type from file data using infer crate, fall back to file extension
    let mime_type = if file_bytes.len() >= MIN_IMAGE_HEADER_SIZE {
        infer::get(&file_bytes).map(|kind| kind.mime_type().to_string())
    } else {
        None
    };

    // If infer couldn't detect, fall back to file extension
    let mime_type = mime_type.unwrap_or_else(|| {
        from_path(attachment_path)
            .first_or_octet_stream()
            .to_string()
    });

    if file_bytes.len() > MAX_ATTACHMENT_SIZE_BYTES {
        return Err(anyhow::anyhow!(
            "File is too large ({}MB). Maximum size is 10MB.",
            file_bytes.len() / (1024 * 1024)
        ));
    }

    let base64_data = general_purpose::STANDARD.encode(&file_bytes);

    let file_name = attachment_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("task_attachment_{index}"));

    Ok(AttachmentInput {
        file_name,
        mime_type,
        data: base64_data,
    })
}

#[cfg(test)]
#[path = "attachments_tests.rs"]
mod tests;
