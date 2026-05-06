//! openWarp: codebase indexing 整模块已弃用,这是过渡期 stub。
//! 公共 API surface 保留为 no-op,所有调用方仍能编译。
//! Phase 4 后续 commit 会逐个删除 caller,最后整模块删干净。

use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use warpui::{AppContext, Entity, ModelContext, ModelHandle, SingletonEntity};

use crate::index::locations::CodeContextLocation;
use crate::workspace::{WorkspaceMetadata, WorkspaceMetadataEvent};

// =============================================================================
// 顶层占位类型
// =============================================================================

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct SyncTask;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ContentHash;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct NodeHash;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct RetrievalID;

#[derive(Debug, Clone, Default)]
pub struct Fragment;

#[derive(Debug, Clone, Default)]
pub struct FragmentLocation;

#[derive(Debug, Clone, Copy, Default)]
pub struct EmbeddingConfig;

#[derive(Debug, Clone, Default)]
pub struct RepoMetadata;

#[derive(Debug, Clone, Copy, Default)]
pub struct CodebaseContextConfig {
    pub embedding_config: EmbeddingConfig,
    pub embedding_cadence: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct CodebaseIndex;

#[derive(Debug, Clone)]
pub enum SyncProgress {
    Discovering {
        total_nodes: usize,
    },
    Syncing {
        completed_nodes: usize,
        total_nodes: usize,
    },
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("openWarp stub: codebase indexing disabled")]
    Disabled,
}

#[derive(Error, Debug)]
pub enum DiffMerkleTreeError {
    #[error("openWarp stub: codebase indexing disabled")]
    Disabled,
}

#[derive(Error, Debug)]
pub enum InconsistentStateError {
    #[error("openWarp stub: codebase indexing disabled")]
    Disabled,
}

// =============================================================================
// store_client sub-module
// =============================================================================

pub mod store_client {
    use super::{ContentHash, NodeHash};

    #[derive(Debug, Clone, Default)]
    pub struct IntermediateNode;

    #[async_trait::async_trait]
    pub trait StoreClient: Send + Sync {
        async fn ping(&self) -> bool {
            false
        }
    }

    /// Empty no-op store client used by the stub.
    pub struct NoopStoreClient;

    #[async_trait::async_trait]
    impl StoreClient for NoopStoreClient {
        async fn ping(&self) -> bool {
            false
        }
    }

    #[allow(dead_code)]
    fn _hash_marker() -> (NodeHash, ContentHash) {
        (NodeHash, ContentHash)
    }
}

// =============================================================================
// manager sub-module
// =============================================================================

pub mod manager {
    use super::*;

    #[derive(Debug, Clone)]
    pub enum CodebaseIndexFinishedStatus {
        Failed(CodebaseIndexingError),
        Succeeded,
    }

    #[derive(Debug, Clone, Error)]
    pub enum CodebaseIndexingError {
        #[error("Exceeded max file limit")]
        ExceededMaxFileLimit,
        #[error("Max depth exceeded")]
        MaxDepthExceeded,
        #[error("openWarp stub: indexing disabled")]
        Disabled,
    }

    #[derive(Debug, Clone, Error)]
    pub enum RetrieveFileError {
        #[error("openWarp stub: indexing disabled")]
        Disabled,
    }

    #[derive(Debug, Clone)]
    pub enum CodebaseIndexManagerEvent {
        IndexMetadataUpdated {
            root_path: PathBuf,
            event: WorkspaceMetadataEvent,
        },
        NewIndexCreated,
        RemoveExpiredIndexMetadata {
            expired_metadata: Vec<WorkspaceMetadata>,
        },
        RetrievalRequestCompleted {
            retrieval_id: RetrievalID,
            fragments: Arc<HashSet<CodeContextLocation>>,
            out_of_sync_delay: Duration,
        },
        RetrievalRequestFailed {
            retrieval_id: RetrievalID,
            error_message: String,
        },
        SyncStateUpdated,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CodebaseIndexStatus;

    impl CodebaseIndexStatus {
        pub fn has_pending(&self) -> bool {
            false
        }
        pub fn has_synced_version(&self) -> bool {
            false
        }
        pub fn last_sync_successful(&self) -> Option<bool> {
            None
        }
        pub fn last_sync_result(&self) -> Option<&CodebaseIndexFinishedStatus> {
            None
        }
        pub fn sync_progress(&self) -> Option<&SyncProgress> {
            None
        }
    }

    pub enum BuildSource<'a> {
        Path(&'a Path),
        _Phantom(PhantomData<&'a ()>),
    }

    /// stub: 始终为空,所有方法 no-op。
    #[derive(Default)]
    pub struct CodebaseIndexManager;

    impl CodebaseIndexManager {
        pub fn new(
            _indices_to_restore: Vec<WorkspaceMetadata>,
            _max_indices_allowed: usize,
            _max_files_per_repo: usize,
            _embedding_generation_batch_size: usize,
            _store_client: Arc<dyn store_client::StoreClient>,
            _ctx: &mut ModelContext<Self>,
        ) -> Self {
            Self
        }

        pub fn new_for_test(
            _store_client: Arc<dyn store_client::StoreClient>,
            _ctx: &mut ModelContext<Self>,
        ) -> Self {
            Self
        }

        pub fn clean_up_deleted_indices(&mut self, _ctx: &mut ModelContext<Self>) {}
        pub fn drop_index(&mut self, _root_path: PathBuf, _ctx: &mut ModelContext<Self>) {}
        pub fn handle_active_session_changed(&mut self, _active_directory: &Path) {}
        pub fn handle_session_bootstrapped(&mut self, _working_directory: &Path) {}
        pub fn has_in_progress_scan(&self) -> bool {
            false
        }
        pub fn update_max_limits(
            &mut self,
            _max_indices_allowed: usize,
            _max_files_per_repo: usize,
            _embedding_generation_batch_size: usize,
        ) {
        }
        pub fn can_create_new_indices(&self) -> bool {
            false
        }
        pub fn get_codebase_index_statuses<'a>(
            &'a self,
            _ctx: &'a AppContext,
        ) -> Vec<(PathBuf, CodebaseIndexStatus)> {
            Vec::new()
        }
        pub fn get_codebase_index_status_for_path<'a>(
            &'a self,
            _path: &Path,
            _ctx: &'a AppContext,
        ) -> Option<CodebaseIndexStatus> {
            None
        }
        pub fn get_codebase_paths(&self) -> impl Iterator<Item = &PathBuf> {
            std::iter::empty()
        }
        pub fn num_active_indices(&self) -> usize {
            0
        }
        pub fn index_directory(&mut self, _directory: PathBuf, _ctx: &mut ModelContext<Self>) {}
        pub fn build_and_sync_codebase_index(
            &mut self,
            _root_path: PathBuf,
            _ctx: &mut ModelContext<Self>,
        ) {
        }
        pub fn reset_codebase_indexing(&mut self, _ctx: &mut ModelContext<Self>) {}
        pub fn root_path_for_codebase(&self, _path: &Path) -> Option<PathBuf> {
            None
        }
        pub fn try_manual_resync_codebase(&self, _repo_path: &Path, _ctx: &mut ModelContext<Self>) {
        }
        pub fn retrieve_relevant_files(
            &mut self,
            _query: String,
            _root_path: &Path,
            _ctx: &mut ModelContext<Self>,
        ) -> Result<RetrievalID, Error> {
            Err(Error::Disabled)
        }
        pub fn abort_retrieval_request(
            &mut self,
            _root_path: &Path,
            _retrieval_id: RetrievalID,
            _ctx: &mut ModelContext<Self>,
        ) -> Result<(), Error> {
            Ok(())
        }
        pub fn write_snapshot(&mut self, _working_directory: &Path, _ctx: &mut ModelContext<Self>) {
        }
        pub fn trigger_incremental_sync_for_path(
            &mut self,
            _path: &Path,
            _ctx: &mut ModelContext<Self>,
        ) {
        }
        pub fn schedule_next_scan(&mut self, _ctx: &mut ModelContext<Self>) {}
    }

    impl Entity for CodebaseIndexManager {
        type Event = CodebaseIndexManagerEvent;
    }

    impl SingletonEntity for CodebaseIndexManager {}

    // 便利:让 ModelHandle<CodebaseIndexManager> 可被 update/as_ref 调用
    #[allow(dead_code)]
    fn _model_handle_marker() -> Option<ModelHandle<CodebaseIndexManager>> {
        None
    }

    // 让某些已使用 HashMap<PathBuf, CodebaseIndexStatus> 的代码继续可用
    #[allow(dead_code)]
    fn _statuses_marker(_m: HashMap<PathBuf, CodebaseIndexStatus>) {}
}

#[allow(dead_code)]
fn _retrieval_marker() -> Option<RetrievalID> {
    None
}
