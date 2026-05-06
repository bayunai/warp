use ai::index::locations::CodeContextLocation;
use anyhow::anyhow;
use futures_util::stream::AbortHandle;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};
use warpui::{AppContext, Entity, ModelContext, SingletonEntity};

use crate::{
    ai::{
        agent::AIAgentActionId,
        outline::{OutlineStatus, RepoOutlines},
    },
    report_error,
    server::server_api::AIApiError,
};

#[derive(Debug)]
pub enum GetRelevantFilesControllerEvent {
    Success {
        action_id: AIAgentActionId,
        fragments: Arc<HashSet<CodeContextLocation>>,
    },
    Error {
        action_id: AIAgentActionId,
    },
}

impl GetRelevantFilesControllerEvent {
    pub fn action_id(&self) -> &AIAgentActionId {
        match self {
            GetRelevantFilesControllerEvent::Success { action_id, .. } => action_id,
            GetRelevantFilesControllerEvent::Error { action_id } => action_id,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetRelevantFilesError {
    #[error("Repo outline is still being computed.")]
    Pending,
    #[error("Failed to create outline.")]
    CreateFailed,
    #[error("Failed to create outline.")]
    Missing,
}

enum RequestHandle {
    AbortHandle(AbortHandle),
}

impl RequestHandle {
    fn abort(&mut self, _ctx: &mut AppContext) {
        match self {
            RequestHandle::AbortHandle(abort_handle) => abort_handle.abort(),
        }
    }
}

/// Controller for GetRelevantFiles action. This is scoped per terminal session.
#[derive(Default)]
pub struct GetRelevantFilesController {
    /// Search requests currently in flight, keyed by the originating action ID.
    /// This allows several SearchCodebase actions to be active at once without newer requests
    /// cancelling unrelated older ones.
    pending_requests: std::collections::HashMap<AIAgentActionId, RequestHandle>,
}

impl GetRelevantFilesController {
    pub fn new(_ctx: &mut ModelContext<Self>) -> Self {
        Self::default()
    }

    /// Start a new search query based on the repo outline.
    pub fn send_request(
        &mut self,
        directory: &Path,
        query: String,
        partial_path_segments: Option<&Vec<String>>,
        action_id: AIAgentActionId,
        ctx: &mut ModelContext<Self>,
    ) -> Result<(), GetRelevantFilesError> {
        const MINIMUM_FILE_COUNT_FOR_API_CALL: usize = 2;
        self.cancel_request_for_action(&action_id, ctx);

        match RepoOutlines::as_ref(ctx).get_outline(directory) {
            Some((OutlineStatus::Complete(outline), base_path)) => {
                let file_outlines = outline.to_file_symbols(partial_path_segments);
                if file_outlines.len() < MINIMUM_FILE_COUNT_FOR_API_CALL {
                    ctx.emit(GetRelevantFilesControllerEvent::Success {
                        action_id,
                        fragments: Arc::new(
                            file_outlines
                                .into_iter()
                                .map(|file| {
                                    CodeContextLocation::WholeFile(PathBuf::from(file.path))
                                })
                                .collect(),
                        ),
                    });
                } else {
                    // BYOP 路径:替换 ServerApi::get_relevant_files 为 BYOP one-shot completion。
                    // OpenWarp 已剥云,无 BYOP 配置时静默 fallback 为整个 outline 当作 relevant
                    // (与原 server 端 ranker 失败时的"全量返回"行为一致,不丢上下文)。
                    use crate::ai::agent_providers::active_ai::relevant_files;
                    let entries: Vec<relevant_files::FileEntry> = file_outlines
                        .iter()
                        .map(|outline| relevant_files::FileEntry {
                            path: outline.path.clone(),
                            symbols: outline.symbols.clone(),
                        })
                        .collect();
                    let action_id_clone = action_id.clone();
                    let Some(prepared) = relevant_files::dispatch(
                        ctx,
                        None,
                        relevant_files::Input {
                            query,
                            files: entries,
                        },
                    ) else {
                        // 无 BYOP active_ai 配置:静默把所有文件当作 relevant 返回,
                        // 让下游照常使用(避免漏 context 比误判更安全)。
                        let fragments: Arc<HashSet<CodeContextLocation>> = Arc::new(
                            file_outlines
                                .into_iter()
                                .map(|file| {
                                    CodeContextLocation::WholeFile(base_path.join(&file.path))
                                })
                                .collect(),
                        );
                        ctx.emit(GetRelevantFilesControllerEvent::Success {
                            action_id,
                            fragments,
                        });
                        return Ok(());
                    };
                    let request_abort_handle = ctx
                        .spawn(
                            async move {
                                let paths = relevant_files::run(prepared).await;
                                Ok::<_, AIApiError>(Arc::new(
                                    paths
                                        .into_iter()
                                        .filter_map(|path| {
                                            let file_path = base_path.join(path);
                                            if file_path.exists() {
                                                Some(CodeContextLocation::WholeFile(file_path))
                                            } else {
                                                None
                                            }
                                        })
                                        .collect(),
                                ))
                            },
                            move |me,
                                  relevant_file_paths: Result<
                                Arc<HashSet<CodeContextLocation>>,
                                AIApiError,
                            >,
                                  ctx| {
                                me.handle_relevant_file_paths_result(
                                    relevant_file_paths.map_err(|e| anyhow!(e)),
                                    action_id_clone,
                                    ctx,
                                )
                            },
                        )
                        .abort_handle();
                    self.pending_requests
                        .insert(action_id, RequestHandle::AbortHandle(request_abort_handle));
                }
                Ok(())
            }
            Some((OutlineStatus::Pending, _)) => Err(GetRelevantFilesError::Pending),
            Some((OutlineStatus::Failed, _)) => Err(GetRelevantFilesError::CreateFailed),
            None => Err(GetRelevantFilesError::Missing),
        }
    }

    fn handle_relevant_file_paths_result(
        &mut self,
        relevant_file_locations: anyhow::Result<Arc<HashSet<CodeContextLocation>>>,
        action_id: AIAgentActionId,
        ctx: &mut ModelContext<Self>,
    ) {
        if self.pending_requests.remove(&action_id).is_none() {
            return;
        }
        match relevant_file_locations {
            Ok(relevant_file_locations) => {
                ctx.emit(GetRelevantFilesControllerEvent::Success {
                    action_id,
                    fragments: relevant_file_locations,
                });
            }
            Err(e) => {
                report_error!(anyhow!(e).context("get_relevant_files failed"));
                ctx.emit(GetRelevantFilesControllerEvent::Error { action_id });
            }
        };
    }

    /// Returns the path to the root directory for a codebase search where pwd is `directory`.
    pub fn root_directory_for_search(&self, directory: &Path, app: &AppContext) -> Option<PathBuf> {
        RepoOutlines::as_ref(app)
            .get_outline(directory)
            .map(|(_, root)| root)
    }

    pub fn cancel_request_for_action(
        &mut self,
        action_id: &AIAgentActionId,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(mut request_handle) = self.pending_requests.remove(action_id) {
            request_handle.abort(ctx);
        }
    }
}

impl Entity for GetRelevantFilesController {
    type Event = GetRelevantFilesControllerEvent;
}
