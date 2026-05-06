use lsp::supported_servers::LSPServerType;
use settings::{
    macros::define_settings_group, RespectUserSyncSetting, SupportedPlatforms, SyncToCloud,
};

define_settings_group!(CodeSettings, settings: [
    code_as_default_editor: CodeAsDefaultEditor {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: false,
        toml_path: "code.editor.use_warp_as_default_editor",
        description: "Whether Warp is used as the default code editor.",
    }

    // openWarp 全局 LSP 启用集合。装好且在此集合中的 server,在任何项目检测到对应语言时自动启动。
    // 不再按 workspace 维持 enablement(替代 PersistedWorkspace.workspaces[*].language_servers)。
    enabled_lsp_servers: EnabledLspServers {
        type: Vec<LSPServerType>,
        default: vec![],
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "code.lsp.enabled_servers",
        description: "Globally enabled LSP servers. Auto-started in any workspace whose language matches.",
    },
    // Whether or not the user has manually dismissed the code toolbelt new feature popup.
    dismissed_code_toolbelt_new_feature_popup: DismissedCodeToolbeltNewFeaturePopup {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: true,
    },
    // Controls whether the project explorer / file tree appears in the tools panel.
    show_project_explorer: ShowProjectExplorer {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "code.editor.show_project_explorer",
        description: "Whether the project explorer is shown in the tools panel.",
    },
    // Controls whether global file search appears in the tools panel.
    show_global_search: ShowGlobalSearch {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "code.editor.show_global_search",
        description: "Whether global file search is shown in the tools panel.",
    },
]);
