use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use pathfinder_geometry::vector::Vector2F;
use remote_server::client::RemoteServerClient;
use remote_server::proto::{
    create_directory_response, list_directory_response, read_file_chunk_response,
    resolve_path_response, write_file_chunk_response, FileSystemEntryKind,
};
use walkdir::WalkDir;
use warp_completer::completer::CommandExitStatus;
use warp_core::ui::theme::color::internal_colors;
use warp_core::HostId;
use warp_util::standardized_path::StandardizedPath;
use warpui::clipboard::ClipboardContent;
use warpui::elements::{
    ChildAnchor, ChildView, ConstrainedBox, Container, CornerRadius, CrossAxisAlignment,
    DispatchEventResult, Element, Empty, EventHandler, Flex, Hoverable, MainAxisSize,
    MouseStateHandle, OffsetPositioning, ParentAnchor, ParentElement, ParentOffsetBounds, Radius,
    SavePosition, ScrollStateHandle, Scrollable, ScrollableElement, ScrollbarWidth, Shrinkable,
    Stack, Text, UniformList, UniformListState,
};
use warpui::platform::{Cursor, FilePickerConfiguration, SaveFilePickerConfiguration};
use warpui::ui_components::components::{Coords, UiComponent, UiComponentStyles};
use warpui::{
    AppContext, Entity, FocusContext, SingletonEntity, TypedActionView, View, ViewContext,
    ViewHandle,
};

use crate::code::buffer_location::RemotePath;
use crate::editor::{
    EditorView, Event as EditorEvent, PropagateAndNoOpNavigationKeys,
    PropagateHorizontalNavigationKeys, SingleLineEditorOptions, TextOptions,
};
use crate::menu::{Menu, MenuItem, MenuItemFields};
use crate::remote_server::manager::RemoteServerManager;
use crate::terminal::model::session::{ExecuteCommandOptions, Session};
use crate::ui_components::icons::Icon;

const ITEM_FONT_SIZE: f32 = 14.0;
const TOOLBAR_BUTTON_SIZE: f32 = 26.0;
const TOOLBAR_ICON_SIZE: f32 = 14.0;
const ITEM_ICON_SIZE: f32 = 14.0;
const ITEM_PADDING_VERTICAL: f32 = 5.0;
const ITEM_PADDING_HORIZONTAL: f32 = 8.0;
const ITEM_ICON_TEXT_SPACING: f32 = 8.0;
const PANEL_HORIZONTAL_PADDING: f32 = 8.0;
const INPUT_HEIGHT: f32 = 30.0;
const CONTEXT_MENU_POSITION_ID: &str = "server_file_browser_panel_root";
const TRANSFER_CHUNK_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug)]
pub enum ServerFileBrowserAction {
    Refresh,
    JumpToPath,
    ClickEntry(usize),
    OpenEntry(usize),
    ToggleDirectory(String),
    SelectPreviousItem,
    SelectNextItem,
    ExpandSelectedItem,
    CollapseSelectedItem,
    ExecuteSelectedItem,
    OpenContextMenu {
        index: usize,
        position: Vector2F,
    },
    DismissContextMenu,
    CopyPath(String),
    JumpToDirectory(String),
    Download(String),
    UploadFiles(String),
    UploadFolder(String),
}

#[derive(Clone, Debug)]
pub enum ServerFileBrowserEvent {
    OpenRemoteFile { remote_path: RemotePath },
}

#[derive(Clone, Debug)]
struct ServerFileBrowserEntry {
    name: String,
    path: String,
    kind: FileSystemEntryKind,
    size_bytes: Option<u64>,
    modified_epoch_millis: Option<u64>,
    depth: usize,
}

pub struct ServerFileBrowserView {
    host_id: Option<HostId>,
    /// Fallback session for executing remote commands when the
    /// remote server daemon is not yet installed / connected.
    session: Option<Arc<Session>>,
    current_path: String,
    path_editor: ViewHandle<EditorView>,
    entries: Vec<ServerFileBrowserEntry>,
    expanded_directories: HashSet<String>,
    loaded_directories: HashMap<String, Vec<ServerFileBrowserEntry>>,
    selected_index: Option<usize>,
    list_state: UniformListState,
    scroll_state: ScrollStateHandle,
    loading: bool,
    status: Option<String>,
    refresh_button: MouseStateHandle,
    upload_file_button: MouseStateHandle,
    upload_folder_button: MouseStateHandle,
    row_states: HashMap<String, MouseStateHandle>,
    context_menu: ViewHandle<Menu<ServerFileBrowserAction>>,
    context_menu_position: Option<Vector2F>,
}

impl ServerFileBrowserView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let context_menu = ctx.add_typed_action_view(|_| {
            Menu::new()
                .prevent_interaction_with_other_elements()
                .with_drop_shadow()
        });
        ctx.subscribe_to_view(&context_menu, |me, _, event, ctx| {
            me.handle_menu_event(event, ctx);
        });

        let path_editor = ctx.add_typed_action_view(|ctx| {
            let appearance = crate::appearance::Appearance::as_ref(ctx);
            let mut editor = EditorView::single_line(
                SingleLineEditorOptions {
                    text: TextOptions::ui_text(Some(ITEM_FONT_SIZE), appearance),
                    select_all_on_focus: true,
                    clear_selections_on_blur: true,
                    propagate_and_no_op_vertical_navigation_keys:
                        PropagateAndNoOpNavigationKeys::Always,
                    propagate_horizontal_navigation_keys: PropagateHorizontalNavigationKeys::Always,
                    ..Default::default()
                },
                ctx,
            );
            editor.set_placeholder_text(crate::t!("server-file-browser-path-placeholder"), ctx);
            editor
        });

        ctx.subscribe_to_view(&path_editor, |me, _, event, ctx| match event {
            EditorEvent::Enter => me.jump_to_editor_path(ctx),
            EditorEvent::Escape => me.sync_editor_to_current_path(ctx),
            _ => {}
        });

        Self {
            host_id: None,
            session: None,
            current_path: String::new(),
            path_editor,
            entries: Vec::new(),
            expanded_directories: HashSet::new(),
            loaded_directories: HashMap::new(),
            selected_index: None,
            list_state: UniformListState::new(),
            scroll_state: ScrollStateHandle::default(),
            loading: false,
            status: Some(crate::t!("server-file-browser-empty")),
            refresh_button: Default::default(),
            upload_file_button: Default::default(),
            upload_folder_button: Default::default(),
            row_states: HashMap::new(),
            context_menu,
            context_menu_position: None,
        }
    }

    pub fn set_remote_root(
        &mut self,
        host_id: HostId,
        path: String,
        session: Option<Arc<Session>>,
        ctx: &mut ViewContext<Self>,
    ) {
        let session_changed = match (&self.session, &session) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };
        let should_load = self.host_id.as_ref() != Some(&host_id)
            || self.current_path != path
            || session_changed;
        self.host_id = Some(host_id);
        self.session = session;
        if should_load {
            self.current_path = path;
            self.sync_editor_to_current_path(ctx);
            self.load_current_directory(ctx);
        }
    }

    pub fn on_left_panel_focused(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.focus_self();
        if self.selected_index.is_none() && !self.entries.is_empty() {
            self.selected_index = Some(0);
        }
        ctx.notify();
    }

    fn sync_editor_to_current_path(&mut self, ctx: &mut ViewContext<Self>) {
        self.path_editor.update(ctx, |editor, ctx| {
            editor.set_buffer_text(&self.current_path, ctx);
        });
    }

    fn jump_to_editor_path(&mut self, ctx: &mut ViewContext<Self>) {
        let path = self.path_editor.as_ref(ctx).buffer_text(ctx).trim().to_string();
        if path.is_empty() {
            return;
        }
        self.resolve_and_open(path, ctx);
    }

    fn client(&self, ctx: &AppContext) -> Option<Arc<RemoteServerClient>> {
        let host_id = self.host_id.as_ref()?;
        RemoteServerManager::as_ref(ctx).client_for_host(host_id).cloned()
    }

    fn set_error(&mut self, message: impl Into<String>, ctx: &mut ViewContext<Self>) {
        self.loading = false;
        self.status = Some(message.into());
        ctx.notify();
    }

    fn load_current_directory(&mut self, ctx: &mut ViewContext<Self>) {
        let path = if self.current_path.is_empty() {
            "~".to_string()
        } else {
            self.current_path.clone()
        };

        let path_for_spawn = path.clone();

        if let Some(client) = self.client(ctx) {
            self.loading = true;
            self.status = None;
            ctx.notify();
            ctx.spawn(
                async move { list_directory(client, path_for_spawn).await },
                |me, result, ctx| {
                    me.loading = false;
                    match result {
                        Ok((canonical_path, entries)) => {
                            me.current_path = canonical_path;
                            me.sync_editor_to_current_path(ctx);
                            me.reset_tree_state();
                            me.entries = entries;
                            me.sync_row_states();
                            me.status = None;
                        }
                        Err(error) => {
                            me.status = Some(error);
                        }
                    }
                    ctx.notify();
                },
            );
        } else if let Some(session) = self.session.clone() {
            self.loading = true;
            self.status = None;
            ctx.notify();
            ctx.spawn(
                async move { list_directory_via_session(session, path_for_spawn).await },
                |me, result, ctx| {
                    me.loading = false;
                    match result {
                        Ok((canonical_path, entries)) => {
                            me.current_path = canonical_path;
                            me.sync_editor_to_current_path(ctx);
                            me.reset_tree_state();
                            me.entries = entries;
                            me.sync_row_states();
                            me.status = None;
                        }
                        Err(error) => {
                            me.status = Some(error);
                        }
                    }
                    ctx.notify();
                },
            );
        } else {
            self.set_error(crate::t!("server-file-browser-no-session"), ctx);
        }
    }

    fn resolve_and_open(&mut self, path: String, ctx: &mut ViewContext<Self>) {
        let host_id = self.host_id.clone();
        let path_for_spawn = path.clone();

        if let Some(client) = self.client(ctx) {
            self.loading = true;
            self.status = None;
            ctx.notify();
            ctx.spawn(
                async move { resolve_path(client, path_for_spawn).await },
                move |me, result, ctx| {
                    me.finish_resolve_and_open(result, host_id, ctx);
                },
            );
        } else if let Some(session) = self.session.clone() {
            self.loading = true;
            self.status = None;
            ctx.notify();
            ctx.spawn(
                async move { resolve_path_via_session(session, path_for_spawn).await },
                move |me, result, ctx| {
                    me.finish_resolve_and_open(result, host_id, ctx);
                },
            );
        } else {
            self.set_error(crate::t!("server-file-browser-no-session"), ctx);
        }
    }

    fn finish_resolve_and_open(
        &mut self,
        result: Result<ResolvedRemotePath, String>,
        host_id: Option<HostId>,
        ctx: &mut ViewContext<Self>,
    ) {
        self.loading = false;
        match result {
            Ok(resolved) if resolved.kind == FileSystemEntryKind::Directory => {
                self.current_path = resolved.canonical_path;
                self.sync_editor_to_current_path(ctx);
                self.load_current_directory(ctx);
            }
            Ok(resolved) if resolved.kind == FileSystemEntryKind::File => {
                if let (Some(host_id), Ok(path)) = (
                    host_id.clone(),
                    StandardizedPath::try_new(&resolved.canonical_path),
                ) {
                    ctx.emit(ServerFileBrowserEvent::OpenRemoteFile {
                        remote_path: RemotePath::new(host_id, path),
                    });
                }
                if let Some(parent) = remote_parent(&resolved.canonical_path) {
                    self.current_path = parent;
                    self.sync_editor_to_current_path(ctx);
                    self.load_current_directory(ctx);
                }
            }
            Ok(_) => {
                self.status = Some(crate::t!("server-file-browser-unsupported-path"));
            }
            Err(error) => {
                self.status = Some(error);
            }
        }
        ctx.notify();
    }

    fn reset_tree_state(&mut self) {
        self.expanded_directories.clear();
        self.loaded_directories.clear();
        self.selected_index = None;
        self.list_state = UniformListState::new();
        self.scroll_state = ScrollStateHandle::default();
        self.context_menu_position = None;
        self.row_states.clear();
    }

    fn sync_row_states(&mut self) {
        let active_paths: HashSet<String> = self.entries.iter().map(|entry| entry.path.clone()).collect();
        for path in &active_paths {
            self.row_states.entry(path.clone()).or_default();
        }
        self.row_states.retain(|path, _| active_paths.contains(path));
    }

    fn toggle_directory(&mut self, path: String, ctx: &mut ViewContext<Self>) {
        if self.expanded_directories.remove(&path) {
            self.rebuild_entries();
            ctx.notify();
            return;
        }

        let child_depth = self
            .entries
            .iter()
            .find(|entry| entry.path == path)
            .map(|entry| entry.depth + 1)
            .unwrap_or(1);
        self.expanded_directories.insert(path.clone());
        if self.loaded_directories.contains_key(&path) {
            self.rebuild_entries();
            ctx.notify();
            return;
        }

        let path_for_spawn = path.clone();
        if let Some(client) = self.client(ctx) {
            self.loading = true;
            ctx.notify();
            ctx.spawn(
                async move { list_directory(client, path_for_spawn).await },
                move |me, result, ctx| {
                    me.loading = false;
                    match result {
                        Ok((path, entries)) => {
                            let entries = entries_with_depth(entries, child_depth);
                            me.loaded_directories.insert(path, entries);
                            me.rebuild_entries();
                        }
                        Err(error) => me.status = Some(error),
                    }
                    ctx.notify();
                },
            );
        } else if let Some(session) = self.session.clone() {
            self.loading = true;
            ctx.notify();
            ctx.spawn(
                async move { list_directory_via_session(session, path_for_spawn).await },
                move |me, result, ctx| {
                    me.loading = false;
                    match result {
                        Ok((path, entries)) => {
                            let entries = entries_with_depth(entries, child_depth);
                            me.loaded_directories.insert(path, entries);
                            me.rebuild_entries();
                        }
                        Err(error) => me.status = Some(error),
                    }
                    ctx.notify();
                },
            );
        } else {
            self.set_error(crate::t!("server-file-browser-no-session"), ctx);
        }
    }

    fn rebuild_entries(&mut self) {
        let selected_path = self
            .selected_index
            .and_then(|index| self.entries.get(index))
            .map(|entry| entry.path.clone());
        let roots = self.entries.iter().filter(|entry| entry.depth == 0).cloned().collect();
        self.entries =
            rebuild_entries_from(roots, &self.expanded_directories, &self.loaded_directories);
        self.selected_index =
            selected_index_after_rebuild(&self.entries, selected_path.as_deref(), self.selected_index);
        self.sync_row_states();
    }

    fn select_index(&mut self, index: usize, ctx: &mut ViewContext<Self>) {
        if index < self.entries.len() {
            self.selected_index = Some(index);
            ctx.notify();
        }
    }

    fn click_entry(&mut self, index: usize, ctx: &mut ViewContext<Self>) {
        let Some(entry) = self.entries.get(index).cloned() else {
            return;
        };
        self.selected_index = Some(index);
        if entry.kind == FileSystemEntryKind::Directory {
            self.toggle_directory(entry.path, ctx);
        } else {
            ctx.notify();
        }
    }

    fn open_index(&mut self, index: usize, ctx: &mut ViewContext<Self>) {
        let Some(entry) = self.entries.get(index).cloned() else {
            return;
        };
        self.selected_index = Some(index);
        match entry.kind {
            FileSystemEntryKind::Directory => self.toggle_directory(entry.path, ctx),
            FileSystemEntryKind::File => {
                if let (Some(host_id), Ok(path)) = (
                    self.host_id.clone(),
                    StandardizedPath::try_new(entry.path.as_str()),
                ) {
                    ctx.emit(ServerFileBrowserEvent::OpenRemoteFile {
                        remote_path: RemotePath::new(host_id, path),
                    });
                }
                ctx.notify();
            }
            FileSystemEntryKind::Symlink
            | FileSystemEntryKind::Other
            | FileSystemEntryKind::Unspecified => {
                self.resolve_and_open(entry.path, ctx);
            }
        }
    }

    fn open_context_menu(&mut self, index: usize, position: Vector2F, ctx: &mut ViewContext<Self>) {
        let Some(entry) = self.entries.get(index).cloned() else {
            return;
        };
        self.selected_index = Some(index);
        self.context_menu_position = Some(position);
        let menu_items = self.context_menu_items(&entry);
        self.context_menu.update(ctx, move |menu, ctx| {
            menu.set_items(menu_items, ctx);
            ctx.notify();
        });
        ctx.notify();
    }

    fn dismiss_context_menu(&mut self, ctx: &mut ViewContext<Self>) {
        self.context_menu_position = None;
        ctx.notify();
    }

    fn handle_menu_event(&mut self, event: &crate::menu::Event, ctx: &mut ViewContext<Self>) {
        if let crate::menu::Event::Close { .. } = event {
            self.context_menu_position = None;
        }
        ctx.notify();
    }

    fn copy_path(&mut self, path: String, ctx: &mut ViewContext<Self>) {
        ctx.clipboard()
            .write(ClipboardContent::plain_text(path.clone()));
        self.status = Some(crate::t!("server-file-browser-copied-path"));
        self.dismiss_context_menu(ctx);
    }

    fn select_previous_item(&mut self, ctx: &mut ViewContext<Self>) {
        self.selected_index = previous_index(self.selected_index, self.entries.len());
        ctx.notify();
    }

    fn select_next_item(&mut self, ctx: &mut ViewContext<Self>) {
        self.selected_index = next_index(self.selected_index, self.entries.len());
        ctx.notify();
    }

    fn expand_selected_item(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(entry) = self
            .selected_index
            .and_then(|index| self.entries.get(index))
            .cloned()
        else {
            return;
        };
        if entry.kind == FileSystemEntryKind::Directory
            && !self.expanded_directories.contains(&entry.path)
        {
            self.toggle_directory(entry.path, ctx);
        }
    }

    fn collapse_selected_item(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(entry) = self
            .selected_index
            .and_then(|index| self.entries.get(index))
            .cloned()
        else {
            return;
        };
        if entry.kind == FileSystemEntryKind::Directory
            && self.expanded_directories.contains(&entry.path)
        {
            self.toggle_directory(entry.path, ctx);
        }
    }

    fn execute_selected_item(&mut self, ctx: &mut ViewContext<Self>) {
        if let Some(index) = self.selected_index {
            self.open_index(index, ctx);
        }
    }

    fn context_menu_items(
        &self,
        target: &ServerFileBrowserEntry,
    ) -> Vec<MenuItem<ServerFileBrowserAction>> {
        let target_is_directory = target.kind == FileSystemEntryKind::Directory;
        let upload_target = if target_is_directory {
            target.path.clone()
        } else {
            remote_parent(&target.path).unwrap_or_else(|| self.current_path.clone())
        };
        let mut items = vec![
            MenuItemFields::new(crate::t!("server-file-browser-menu-download"))
                .with_on_select_action(ServerFileBrowserAction::Download(target.path.clone()))
                .into_item(),
            MenuItemFields::new(crate::t!("server-file-browser-menu-upload-file"))
                .with_on_select_action(ServerFileBrowserAction::UploadFiles(upload_target.clone()))
                .into_item(),
            MenuItemFields::new(crate::t!("server-file-browser-menu-upload-folder"))
                .with_on_select_action(ServerFileBrowserAction::UploadFolder(upload_target))
                .into_item(),
            MenuItemFields::new(crate::t!("server-file-browser-menu-copy-path"))
                .with_on_select_action(ServerFileBrowserAction::CopyPath(target.path.clone()))
                .into_item(),
        ];
        if target_is_directory {
            items.push(
                MenuItemFields::new(crate::t!("server-file-browser-menu-jump-to-path"))
                    .with_on_select_action(ServerFileBrowserAction::JumpToDirectory(
                        target.path.clone(),
                    ))
                    .into_item(),
            );
        }
        items
    }

    fn choose_and_upload_files(&mut self, remote_directory: String, ctx: &mut ViewContext<Self>) {
        let Some(client) = self.client(ctx) else {
            self.set_error(crate::t!("server-file-browser-no-session"), ctx);
            return;
        };
        ctx.open_file_picker(
            move |result, ctx| match result {
                Ok(paths) if !paths.is_empty() => {
                    ctx.spawn(
                        async move {
                            upload_paths(
                                client,
                                paths.into_iter().map(PathBuf::from).collect(),
                                remote_directory,
                                false,
                            )
                            .await
                        },
                        |me: &mut Self, result, ctx| {
                            me.finish_transfer(result, ctx);
                        },
                    );
                }
                Ok(_) => {}
                Err(error) => {
                    log::warn!("server file browser file picker failed: {error}");
                }
            },
            FilePickerConfiguration::new().allow_multi_select(),
        );
    }

    fn choose_and_upload_folder(&mut self, remote_directory: String, ctx: &mut ViewContext<Self>) {
        let Some(client) = self.client(ctx) else {
            self.set_error(crate::t!("server-file-browser-no-session"), ctx);
            return;
        };
        ctx.open_file_picker(
            move |result, ctx| match result {
                Ok(paths) if !paths.is_empty() => {
                    ctx.spawn(
                        async move {
                            upload_paths(
                                client,
                                paths.into_iter().map(PathBuf::from).collect(),
                                remote_directory,
                                true,
                            )
                            .await
                        },
                        |me: &mut Self, result, ctx| {
                            me.finish_transfer(result, ctx);
                        },
                    );
                }
                Ok(_) => {}
                Err(error) => {
                    log::warn!("server file browser folder picker failed: {error}");
                }
            },
            FilePickerConfiguration::new().folders_only(),
        );
    }

    fn choose_download_destination(&mut self, remote_path: String, ctx: &mut ViewContext<Self>) {
        let Some(entry) = self.entries.iter().find(|entry| entry.path == remote_path).cloned() else {
            return;
        };
        let Some(client) = self.client(ctx) else {
            self.set_error(crate::t!("server-file-browser-no-session"), ctx);
            return;
        };

        match entry.kind {
            FileSystemEntryKind::Directory => {
                ctx.open_file_picker(
                    move |result, ctx| match result {
                        Ok(paths) if !paths.is_empty() => {
                            let destination = PathBuf::from(&paths[0]);
                            ctx.spawn(
                                async move { download_directory(client, entry.path, destination).await },
                                |me: &mut Self, result, ctx| {
                                    me.finish_transfer(result, ctx);
                                },
                            );
                        }
                        Ok(_) => {}
                        Err(error) => {
                            log::warn!("server file browser download picker failed: {error}");
                        }
                    },
                    FilePickerConfiguration::new().folders_only(),
                );
            }
            _ => {
                let default_filename = remote_basename(&entry.path).unwrap_or(entry.name);
                ctx.open_save_file_picker(
                    move |path, _me, ctx| {
                        if let Some(path) = path {
                            ctx.spawn(
                                async move {
                                    download_file(client, entry.path, PathBuf::from(path)).await
                                },
                                |me: &mut Self, result, ctx| {
                                    me.finish_transfer(result, ctx);
                                },
                            );
                        }
                    },
                    SaveFilePickerConfiguration::new().with_default_filename(default_filename),
                );
            }
        }
    }

    fn finish_transfer(&mut self, result: Result<(), String>, ctx: &mut ViewContext<Self>) {
        match result {
            Ok(()) => {
                self.status = Some(crate::t!("server-file-browser-transfer-complete"));
                self.load_current_directory(ctx);
            }
            Err(error) => {
                self.status = Some(error);
                ctx.notify();
            }
        }
    }

    fn render_toolbar(&self, appearance: &crate::appearance::Appearance) -> Box<dyn Element> {
        let theme = appearance.theme();
        let icon_color = theme.sub_text_color(theme.background());
        let make_btn =
            |icon: Icon, state: MouseStateHandle, action: ServerFileBrowserAction| -> Box<dyn Element> {
                let icon_el = ConstrainedBox::new(icon.to_warpui_icon(icon_color).finish())
                    .with_width(TOOLBAR_ICON_SIZE)
                    .with_height(TOOLBAR_ICON_SIZE)
                    .finish();
                Hoverable::new(state, move |_| {
                    Container::new(
                        ConstrainedBox::new(icon_el)
                            .with_width(TOOLBAR_BUTTON_SIZE)
                            .with_height(TOOLBAR_BUTTON_SIZE)
                            .finish(),
                    )
                    .with_uniform_padding(2.0)
                    .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.0)))
                    .finish()
                })
                .with_cursor(Cursor::PointingHand)
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(action.clone());
                })
                .finish()
            };

        Flex::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_spacing(6.0)
            .with_child(Shrinkable::new(
                1.0,
                appearance
                    .ui_builder()
                    .text_input(self.path_editor.clone())
                    .with_style(UiComponentStyles {
                        height: Some(INPUT_HEIGHT),
                        padding: Some(Coords::uniform(6.0)),
                        background: Some(theme.surface_2().into()),
                        border_color: Some(theme.nonactive_ui_detail().into()),
                        border_width: Some(1.0),
                        border_radius: Some(CornerRadius::with_all(Radius::Pixels(4.0))),
                        font_size: Some(ITEM_FONT_SIZE),
                        ..Default::default()
                    })
                    .build()
                    .finish(),
            )
            .finish())
            .with_child(make_btn(
                Icon::Refresh,
                self.refresh_button.clone(),
                ServerFileBrowserAction::Refresh,
            ))
            .with_child(make_btn(
                Icon::UploadCloud,
                self.upload_file_button.clone(),
                ServerFileBrowserAction::UploadFiles(self.current_path.clone()),
            ))
            .with_child(make_btn(
                Icon::Folder,
                self.upload_folder_button.clone(),
                ServerFileBrowserAction::UploadFolder(self.current_path.clone()),
            ))
            .finish()
    }

    fn render_entries(&self, appearance: &crate::appearance::Appearance) -> Box<dyn Element> {
        let theme = appearance.theme();
        if self.host_id.is_none() {
            return self.render_status_text(crate::t!("server-file-browser-no-session"), appearance);
        } else if self.loading && self.entries.is_empty() {
            return self.render_status_text(crate::t!("server-file-browser-loading"), appearance);
        } else if self.entries.is_empty() {
            return self.render_status_text(crate::t!("server-file-browser-empty-directory"), appearance);
        }

        let entries = self.entries.clone();
        let selected_index = self.selected_index;
        let expanded_directories = self.expanded_directories.clone();
        let row_states = self.row_states.clone();
        let uniform_list = UniformList::new(
            self.list_state.clone(),
            entries.len(),
            move |range, app| {
                let appearance = crate::appearance::Appearance::as_ref(app);
                range
                    .filter_map(|index| {
                        let entry = entries.get(index)?;
                        let state = row_states
                            .get(&entry.path)
                            .cloned()
                            .expect("row mouse state is synced before render");
                        Some(render_entry_row(
                            index,
                            entry,
                            selected_index == Some(index),
                            expanded_directories.contains(&entry.path),
                            state,
                            appearance,
                        ))
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
            },
        )
        .finish_scrollable();

        let scrollable = Shrinkable::new(
            1.0,
            Scrollable::vertical(
                self.scroll_state.clone(),
                uniform_list,
                ScrollbarWidth::Auto,
                theme.nonactive_ui_detail().into(),
                theme.active_ui_detail().into(),
                warpui::elements::Fill::None,
            )
            .with_overlayed_scrollbar()
            .finish(),
        )
        .finish();

        let mut col = Flex::column()
            .with_main_axis_size(MainAxisSize::Max)
            .with_child(scrollable);
        if let Some(status) = &self.status {
            col.add_child(
                Container::new(
                    Text::new_inline(status.clone(), appearance.ui_font_family(), 12.0)
                        .with_color(theme.sub_text_color(theme.background()).into())
                        .finish(),
                )
                .with_padding_top(10.0)
                .with_padding_left(ITEM_PADDING_HORIZONTAL)
                .with_padding_right(ITEM_PADDING_HORIZONTAL)
                .finish(),
            );
        }

        let content = Container::new(
            col.with_cross_axis_alignment(CrossAxisAlignment::Stretch)
                .finish(),
        )
        .with_horizontal_padding(PANEL_HORIZONTAL_PADDING - ITEM_PADDING_HORIZONTAL);

        content.finish()
    }

    fn render_status_text(
        &self,
        text: String,
        appearance: &crate::appearance::Appearance,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        Container::new(
            Text::new_inline(text, appearance.ui_font_family(), ITEM_FONT_SIZE)
                .with_color(theme.sub_text_color(theme.background()).into())
                .finish(),
        )
        .with_padding_top(20.0)
        .with_padding_bottom(20.0)
        .with_padding_left(ITEM_PADDING_HORIZONTAL)
        .with_padding_right(ITEM_PADDING_HORIZONTAL)
        .finish()
    }

}

fn render_entry_row(
    index: usize,
    entry: &ServerFileBrowserEntry,
    is_selected: bool,
    is_expanded: bool,
    state: MouseStateHandle,
    appearance: &crate::appearance::Appearance,
) -> Box<dyn Element> {
    let theme = appearance.theme();
    let icon_color = theme.sub_text_color(theme.background());
    let is_directory = entry.kind == FileSystemEntryKind::Directory;

    let chevron: Box<dyn Element> = if is_directory {
        let icon = if is_expanded {
            Icon::ChevronDown
        } else {
            Icon::ChevronRight
        };
        ConstrainedBox::new(icon.to_warpui_icon(icon_color).finish())
            .with_width(ITEM_ICON_SIZE)
            .with_height(ITEM_ICON_SIZE)
            .finish()
    } else {
        ConstrainedBox::new(Empty::new().finish())
            .with_width(ITEM_ICON_SIZE)
            .finish()
    };
    let icon = if is_directory { Icon::Folder } else { Icon::File };
    let icon_el = ConstrainedBox::new(icon.to_warpui_icon(icon_color).finish())
        .with_width(ITEM_ICON_SIZE)
        .with_height(ITEM_ICON_SIZE)
        .finish();
    let label = Text::new_inline(
        entry.name.clone(),
        appearance.ui_font_family(),
        ITEM_FONT_SIZE,
    )
    .with_color(theme.main_text_color(theme.background()).into())
    .finish();

    let mut metadata_parts = Vec::new();
    if let Some(size) = entry.size_bytes {
        metadata_parts.push(format_file_size(size));
    }
    if entry.modified_epoch_millis.is_some() {
        metadata_parts.push(crate::t!("server-file-browser-modified"));
    }
    let metadata = (!metadata_parts.is_empty()).then(|| {
        Text::new_inline(metadata_parts.join(" · "), appearance.ui_font_family(), 11.0)
            .with_color(theme.sub_text_color(theme.background()).into())
            .finish()
    });

    let text_column = {
        let mut col = Flex::column()
            .with_main_axis_size(MainAxisSize::Min)
            .with_child(label);
        if let Some(metadata) = metadata {
            col.add_child(metadata);
        }
        col.finish()
    };

    let row = Flex::row()
        .with_main_axis_size(MainAxisSize::Max)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_spacing(ITEM_ICON_TEXT_SPACING)
        .with_child(
            ConstrainedBox::new(Empty::new().finish())
                .with_width(entry.depth as f32 * 16.0)
                .finish(),
        )
        .with_child(chevron)
        .with_child(icon_el)
        .with_child(Shrinkable::new(1.0, text_column).finish())
        .finish();

    let hoverable = Hoverable::new(state, move |_| {
        let mut container = Container::new(row)
            .with_padding_top(ITEM_PADDING_VERTICAL)
            .with_padding_bottom(ITEM_PADDING_VERTICAL)
            .with_padding_left(ITEM_PADDING_HORIZONTAL)
            .with_padding_right(ITEM_PADDING_HORIZONTAL)
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.0)));
        if is_selected {
            container = container.with_background(internal_colors::fg_overlay_3(theme));
        }
        container.finish()
    })
    .with_cursor(Cursor::PointingHand)
    .on_click(move |ctx, _, _| {
        ctx.dispatch_typed_action(ServerFileBrowserAction::ClickEntry(index));
    })
    .on_double_click(move |ctx, _, _| {
        ctx.dispatch_typed_action(ServerFileBrowserAction::OpenEntry(index));
    })
    .on_right_click(move |ctx, _, position| {
        let offset = match ctx.element_position_by_id(CONTEXT_MENU_POSITION_ID) {
            Some(bounds) => position - bounds.origin(),
            None => position,
        };
        ctx.dispatch_typed_action(ServerFileBrowserAction::OpenContextMenu {
            index,
            position: offset,
        });
    })
    .finish();

    Container::new(hoverable).finish()
}

impl Entity for ServerFileBrowserView {
    type Event = ServerFileBrowserEvent;
}

impl TypedActionView for ServerFileBrowserView {
    type Action = ServerFileBrowserAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            ServerFileBrowserAction::Refresh => self.load_current_directory(ctx),
            ServerFileBrowserAction::JumpToPath => self.jump_to_editor_path(ctx),
            ServerFileBrowserAction::ClickEntry(index) => {
                ctx.focus_self();
                self.click_entry(*index, ctx);
            }
            ServerFileBrowserAction::OpenEntry(index) => {
                ctx.focus_self();
                self.open_index(*index, ctx);
            }
            ServerFileBrowserAction::ToggleDirectory(path) => self.toggle_directory(path.clone(), ctx),
            ServerFileBrowserAction::SelectPreviousItem => self.select_previous_item(ctx),
            ServerFileBrowserAction::SelectNextItem => self.select_next_item(ctx),
            ServerFileBrowserAction::ExpandSelectedItem => self.expand_selected_item(ctx),
            ServerFileBrowserAction::CollapseSelectedItem => self.collapse_selected_item(ctx),
            ServerFileBrowserAction::ExecuteSelectedItem => self.execute_selected_item(ctx),
            ServerFileBrowserAction::OpenContextMenu { index, position } => {
                self.open_context_menu(*index, *position, ctx);
            }
            ServerFileBrowserAction::DismissContextMenu => self.dismiss_context_menu(ctx),
            ServerFileBrowserAction::CopyPath(path) => self.copy_path(path.clone(), ctx),
            ServerFileBrowserAction::JumpToDirectory(path) => {
                self.dismiss_context_menu(ctx);
                self.current_path = path.clone();
                self.sync_editor_to_current_path(ctx);
                self.load_current_directory(ctx);
            }
            ServerFileBrowserAction::Download(path) => {
                self.dismiss_context_menu(ctx);
                self.choose_download_destination(path.clone(), ctx);
            }
            ServerFileBrowserAction::UploadFiles(path) => {
                self.dismiss_context_menu(ctx);
                self.choose_and_upload_files(path.clone(), ctx);
            }
            ServerFileBrowserAction::UploadFolder(path) => {
                self.dismiss_context_menu(ctx);
                self.choose_and_upload_folder(path.clone(), ctx);
            }
        }
    }
}

impl View for ServerFileBrowserView {
    fn ui_name() -> &'static str {
        "ServerFileBrowserView"
    }

    fn on_focus(&mut self, focus_ctx: &FocusContext, ctx: &mut ViewContext<Self>) {
        if focus_ctx.is_self_focused() {
            if self.selected_index.is_none() && !self.entries.is_empty() {
                self.selected_index = Some(0);
                ctx.notify();
            }
        }
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = crate::appearance::Appearance::as_ref(app);
        let toolbar = Container::new(self.render_toolbar(appearance))
            .with_uniform_padding(8.0)
            .finish();
        let entries = Shrinkable::new(1.0, self.render_entries(appearance)).finish();
        let panel = SavePosition::new(
            Container::new(
                Flex::column()
                    .with_main_axis_size(MainAxisSize::Max)
                    .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .with_child(toolbar)
                    .with_child(entries)
                    .finish(),
            )
            .finish(),
            CONTEXT_MENU_POSITION_ID,
        )
        .finish();

        let mut stack = Stack::new();
        stack.add_child(panel);
        if let Some(position) = self.context_menu_position {
            stack.add_positioned_overlay_child(
                ChildView::new(&self.context_menu).finish(),
                OffsetPositioning::offset_from_parent(
                    position,
                    ParentOffsetBounds::ParentByPosition,
                    ParentAnchor::TopLeft,
                    ChildAnchor::TopLeft,
                ),
            );
        }
        EventHandler::new(stack.finish())
            .on_keydown(|ctx, _app, keystroke| {
                match keystroke.normalized().as_str() {
                    "up" => {
                        ctx.dispatch_typed_action(ServerFileBrowserAction::SelectPreviousItem);
                        DispatchEventResult::StopPropagation
                    }
                    "down" => {
                        ctx.dispatch_typed_action(ServerFileBrowserAction::SelectNextItem);
                        DispatchEventResult::StopPropagation
                    }
                    "right" => {
                        ctx.dispatch_typed_action(ServerFileBrowserAction::ExpandSelectedItem);
                        DispatchEventResult::StopPropagation
                    }
                    "left" => {
                        ctx.dispatch_typed_action(ServerFileBrowserAction::CollapseSelectedItem);
                        DispatchEventResult::StopPropagation
                    }
                    "enter" => {
                        ctx.dispatch_typed_action(ServerFileBrowserAction::ExecuteSelectedItem);
                        DispatchEventResult::StopPropagation
                    }
                    "escape" => {
                        ctx.dispatch_typed_action(ServerFileBrowserAction::DismissContextMenu);
                        DispatchEventResult::StopPropagation
                    }
                    _ => DispatchEventResult::PropagateToParent,
                }
            })
            .finish()
    }
}

fn entries_with_depth(
    mut entries: Vec<ServerFileBrowserEntry>,
    depth: usize,
) -> Vec<ServerFileBrowserEntry> {
    for entry in &mut entries {
        entry.depth = depth;
    }
    entries
}

fn rebuild_entries_from(
    entries: Vec<ServerFileBrowserEntry>,
    expanded_directories: &HashSet<String>,
    loaded_directories: &HashMap<String, Vec<ServerFileBrowserEntry>>,
) -> Vec<ServerFileBrowserEntry> {
    let roots = entries
        .into_iter()
        .filter(|entry| entry.depth == 0)
        .collect();
    let mut rebuilt = Vec::new();
    append_entries_from(roots, expanded_directories, loaded_directories, &mut rebuilt);
    rebuilt
}

fn append_entries_from(
    entries: Vec<ServerFileBrowserEntry>,
    expanded_directories: &HashSet<String>,
    loaded_directories: &HashMap<String, Vec<ServerFileBrowserEntry>>,
    out: &mut Vec<ServerFileBrowserEntry>,
) {
    for entry in entries {
        let path = entry.path.clone();
        out.push(entry);
        if expanded_directories.contains(&path) {
            if let Some(children) = loaded_directories.get(&path) {
                append_entries_from(
                    children.clone(),
                    expanded_directories,
                    loaded_directories,
                    out,
                );
            }
        }
    }
}

fn previous_index(selected_index: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some(selected_index.unwrap_or(0).saturating_sub(1))
    }
}

fn next_index(selected_index: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some((selected_index.unwrap_or(0) + 1).min(len - 1))
    }
}

fn selected_index_after_rebuild(
    entries: &[ServerFileBrowserEntry],
    selected_path: Option<&str>,
    fallback_index: Option<usize>,
) -> Option<usize> {
    selected_path
        .and_then(|path| entries.iter().position(|entry| entry.path == path))
        .or_else(|| {
            (!entries.is_empty()).then_some(
                fallback_index
                    .unwrap_or(0)
                    .min(entries.len().saturating_sub(1)),
            )
        })
}

async fn resolve_path(
    client: Arc<RemoteServerClient>,
    path: String,
) -> Result<ResolvedRemotePath, String> {
    let response = client.resolve_path(path).await.map_err(|error| error.to_string())?;
    match response.result {
        Some(resolve_path_response::Result::Success(success)) => {
            let kind = FileSystemEntryKind::try_from(success.kind)
                .unwrap_or(FileSystemEntryKind::Other);
            Ok(ResolvedRemotePath {
                canonical_path: success.canonical_path,
                kind,
            })
        }
        Some(resolve_path_response::Result::Error(error)) => Err(error.message),
        None => Err(crate::t!("server-file-browser-empty-response")),
    }
}

async fn list_directory(
    client: Arc<RemoteServerClient>,
    path: String,
) -> Result<(String, Vec<ServerFileBrowserEntry>), String> {
    let response = client.list_directory(path).await.map_err(|error| error.to_string())?;
    match response.result {
        Some(list_directory_response::Result::Success(success)) => {
            let canonical_path = success.canonical_path;
            let mut entries = Vec::with_capacity(success.entries.len());
            for entry in success.entries {
                let kind =
                    FileSystemEntryKind::try_from(entry.kind).unwrap_or(FileSystemEntryKind::Other);
                entries.push(ServerFileBrowserEntry {
                    path: join_remote_path(&canonical_path, &entry.name),
                    name: entry.name,
                    kind,
                    size_bytes: entry.size_bytes,
                    modified_epoch_millis: entry.modified_epoch_millis,
                    depth: 0,
                });
            }
            Ok((canonical_path, entries))
        }
        Some(list_directory_response::Result::Error(error)) => Err(error.message),
        None => Err(crate::t!("server-file-browser-empty-response")),
    }
}

/// Fallback directory listing via `Session::execute_command` when the
/// remote server daemon is not installed.
async fn list_directory_via_session(
    session: Arc<Session>,
    path: String,
) -> Result<(String, Vec<ServerFileBrowserEntry>), String> {
    let escaped = warp_util::path::ShellFamily::Posix.shell_escape(&path);
    let script = format!(
        "cd {escaped} && find . -maxdepth 1 -type d -print0 && printf '\\000' && find . -maxdepth 1 -not -type d -print0"
    );
    let output = session
        .execute_command(&script, None, None, ExecuteCommandOptions::default())
        .await
        .map_err(|e| format!("{e:#}"))?;

    if output.status != CommandExitStatus::Success {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ls failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.split('\0');
    // Directories come first, separated from files by an empty entry
    // (double null). Find the separator.
    let mut dirs: Vec<&str> = Vec::new();
    let mut files: Vec<&str> = Vec::new();
    let mut found_separator = false;
    for part in parts.by_ref() {
        if part.is_empty() {
            found_separator = true;
            break;
        }
        if part != "." {
            dirs.push(part);
        }
    }
    if found_separator {
        for part in parts {
            if !part.is_empty() {
                files.push(part);
            }
        }
    }

    let mut entries = Vec::with_capacity(dirs.len() + files.len());
    // Canonical path: if the path is relative, use it as-is (the remote
    // host may not support canonicalize).
    let canonical_path = path.clone();
    for name in dirs {
        let name = Path::new(name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(name)
            .to_string();
        entries.push(ServerFileBrowserEntry {
            path: join_remote_path(&canonical_path, &name),
            name,
            kind: FileSystemEntryKind::Directory,
            size_bytes: None,
            modified_epoch_millis: None,
            depth: 0,
        });
    }
    for name in files {
        let name = Path::new(name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(name)
            .to_string();
        entries.push(ServerFileBrowserEntry {
            path: join_remote_path(&canonical_path, &name),
            name,
            kind: FileSystemEntryKind::File,
            size_bytes: None,
            modified_epoch_millis: None,
            depth: 0,
        });
    }
    // Sort alphabetically, directories first.
    entries.sort_by(|a, b| {
        let a_is_dir = a.kind == FileSystemEntryKind::Directory;
        let b_is_dir = b.kind == FileSystemEntryKind::Directory;
        b_is_dir
            .cmp(&a_is_dir)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok((canonical_path, entries))
}

/// Fallback path resolution via `Session::execute_command`.
async fn resolve_path_via_session(
    session: Arc<Session>,
    path: String,
) -> Result<ResolvedRemotePath, String> {
    let escaped = warp_util::path::ShellFamily::Posix.shell_escape(&path);
    // Use a single stat command to determine file type.
    let script = format!(
        "if [ -d {escaped} ]; then echo d; elif [ -f {escaped} ]; then echo f; elif [ -L {escaped} ]; then echo l; else echo o; fi"
    );
    let output = session
        .execute_command(&script, None, None, ExecuteCommandOptions::default())
        .await
        .map_err(|e| format!("{e:#}"))?;

    if output.status != CommandExitStatus::Success {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("stat failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let kind = match stdout.as_str() {
        "d" => FileSystemEntryKind::Directory,
        "f" => FileSystemEntryKind::File,
        "l" => FileSystemEntryKind::Symlink,
        _ => FileSystemEntryKind::Other,
    };

    Ok(ResolvedRemotePath {
        canonical_path: path,
        kind,
    })
}

#[derive(Clone)]
struct ResolvedRemotePath {
    canonical_path: String,
    kind: FileSystemEntryKind,
}

async fn upload_paths(
    client: Arc<RemoteServerClient>,
    local_paths: Vec<PathBuf>,
    remote_directory: String,
    preserve_directory_root: bool,
) -> Result<(), String> {
    for local_path in local_paths {
        if local_path.is_dir() {
            let root_name = local_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "upload".to_string());
            let root_remote = if preserve_directory_root {
                join_remote_path(&remote_directory, &root_name)
            } else {
                remote_directory.clone()
            };
            create_remote_directory(client.clone(), root_remote.clone()).await?;
            for entry in WalkDir::new(&local_path).into_iter().filter_map(Result::ok) {
                let path = entry.path();
                let Ok(relative) = path.strip_prefix(&local_path) else {
                    continue;
                };
                if relative.as_os_str().is_empty() {
                    continue;
                }
                let remote_path = join_remote_path(&root_remote, &relative.to_string_lossy());
                if entry.file_type().is_dir() {
                    create_remote_directory(client.clone(), remote_path).await?;
                } else if entry.file_type().is_file() {
                    upload_file(client.clone(), path.to_path_buf(), remote_path).await?;
                }
            }
        } else if local_path.is_file() {
            let Some(name) = local_path.file_name().map(|name| name.to_string_lossy().to_string())
            else {
                continue;
            };
            let remote_path = join_remote_path(&remote_directory, &name);
            upload_file(client.clone(), local_path, remote_path).await?;
        }
    }
    Ok(())
}

async fn create_remote_directory(
    client: Arc<RemoteServerClient>,
    remote_path: String,
) -> Result<(), String> {
    let response = client
        .create_directory(remote_path)
        .await
        .map_err(|error| error.to_string())?;
    match response.result {
        Some(create_directory_response::Result::Success(_)) => Ok(()),
        Some(create_directory_response::Result::Error(error)) => Err(error.message),
        None => Ok(()),
    }
}

async fn upload_file(
    client: Arc<RemoteServerClient>,
    local_path: PathBuf,
    remote_path: String,
) -> Result<(), String> {
    let bytes = tokio::fs::read(local_path)
        .await
        .map_err(|error| error.to_string())?;
    let mut offset = 0;
    let mut truncate = true;
    for chunk in bytes.chunks(TRANSFER_CHUNK_BYTES as usize) {
        let response = client
            .write_file_chunk(remote_path.clone(), offset, chunk.to_vec(), truncate, None)
            .await
            .map_err(|error| error.to_string())?;
        match response.result {
            Some(write_file_chunk_response::Result::Success(success)) => {
                offset = success.next_offset;
                truncate = false;
            }
            Some(write_file_chunk_response::Result::Error(error)) => return Err(error.message),
            None => return Err(crate::t!("server-file-browser-empty-response")),
        }
    }
    if bytes.is_empty() {
        let response = client
            .write_file_chunk(remote_path, 0, Vec::new(), true, None)
            .await
            .map_err(|error| error.to_string())?;
        if let Some(write_file_chunk_response::Result::Error(error)) = response.result {
            return Err(error.message);
        }
    }
    Ok(())
}

async fn download_file(
    client: Arc<RemoteServerClient>,
    remote_path: String,
    local_path: PathBuf,
) -> Result<(), String> {
    if let Some(parent) = local_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let mut output = tokio::fs::File::create(local_path)
        .await
        .map_err(|error| error.to_string())?;
    let mut offset = 0;
    loop {
        let response = client
            .read_file_chunk(remote_path.clone(), offset, TRANSFER_CHUNK_BYTES)
            .await
            .map_err(|error| error.to_string())?;
        match response.result {
            Some(read_file_chunk_response::Result::Success(success)) => {
                use tokio::io::AsyncWriteExt;
                output
                    .write_all(&success.bytes)
                    .await
                    .map_err(|error| error.to_string())?;
                offset = success.next_offset;
                if success.eof {
                    break;
                }
            }
            Some(read_file_chunk_response::Result::Error(error)) => return Err(error.message),
            None => return Err(crate::t!("server-file-browser-empty-response")),
        }
    }
    Ok(())
}

async fn download_directory(
    client: Arc<RemoteServerClient>,
    remote_path: String,
    local_directory: PathBuf,
) -> Result<(), String> {
    let root_name = remote_basename(&remote_path).unwrap_or_else(|| "download".to_string());
    let root_destination = local_directory.join(root_name);
    tokio::fs::create_dir_all(&root_destination)
        .await
        .map_err(|error| error.to_string())?;
    download_directory_into(client, remote_path, root_destination).await
}

async fn download_directory_into(
    client: Arc<RemoteServerClient>,
    remote_path: String,
    local_directory: PathBuf,
) -> Result<(), String> {
    let (_, entries) = list_directory(client.clone(), remote_path).await?;
    for entry in entries {
        let local_path = local_directory.join(&entry.name);
        match entry.kind {
            FileSystemEntryKind::Directory => {
                tokio::fs::create_dir_all(&local_path)
                    .await
                    .map_err(|error| error.to_string())?;
                Box::pin(download_directory_into(client.clone(), entry.path, local_path)).await?;
            }
            FileSystemEntryKind::File
            | FileSystemEntryKind::Symlink
            | FileSystemEntryKind::Other
            | FileSystemEntryKind::Unspecified => {
                download_file(client.clone(), entry.path, local_path).await?;
            }
        }
    }
    Ok(())
}

fn join_remote_path(base: &str, name: &str) -> String {
    let normalized_name = name.replace('\\', "/");
    if base == "/" {
        format!("/{normalized_name}")
    } else if base.ends_with('/') {
        format!("{base}{normalized_name}")
    } else {
        format!("{base}/{normalized_name}")
    }
}

fn remote_parent(path: &str) -> Option<String> {
    let trimmed = path.trim_end_matches('/');
    let idx = trimmed.rfind('/')?;
    if idx == 0 {
        Some("/".to_string())
    } else {
        Some(trimmed[..idx].to_string())
    }
}

fn remote_basename(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .or_else(|| path.trim_end_matches('/').rsplit('/').next().map(str::to_string))
}

fn format_file_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let size = size as f64;
    if size >= GB {
        format!("{:.1} GB", size / GB)
    } else if size >= MB {
        format!("{:.1} MB", size / MB)
    } else if size >= KB {
        format!("{:.1} KB", size / KB)
    } else {
        format!("{} B", size as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        path: &str,
        name: &str,
        kind: FileSystemEntryKind,
        depth: usize,
    ) -> ServerFileBrowserEntry {
        ServerFileBrowserEntry {
            name: name.to_string(),
            path: path.to_string(),
            kind,
            size_bytes: None,
            modified_epoch_millis: None,
            depth,
        }
    }

    #[test]
    fn rebases_loaded_directory_entries_to_parent_depth() {
        let entries = vec![
            entry(
                "/root/.openwarp/remote-server/warp-oss",
                "warp-oss",
                FileSystemEntryKind::File,
                0,
            ),
            entry(
                "/root/.openwarp/remote-server/logs",
                "logs",
                FileSystemEntryKind::Directory,
                0,
            ),
        ];

        let entries = entries_with_depth(entries, 1);

        assert_eq!(
            entries.iter().map(|entry| entry.depth).collect::<Vec<_>>(),
            vec![1, 1]
        );
    }

    #[test]
    fn rebuild_entries_does_not_promote_loaded_children_to_roots() {
        let root = entry(
            "/root/.openwarp/remote-server",
            "remote-server",
            FileSystemEntryKind::Directory,
            0,
        );
        let child = entry(
            "/root/.openwarp/remote-server/warp-oss",
            "warp-oss",
            FileSystemEntryKind::File,
            0,
        );
        let expanded_directories = HashSet::from([root.path.clone()]);
        let loaded_directories =
            HashMap::from([(root.path.clone(), entries_with_depth(vec![child], 1))]);

        let rebuilt = rebuild_entries_from(
            vec![root.clone()],
            &expanded_directories,
            &loaded_directories,
        );
        let rebuilt_again =
            rebuild_entries_from(rebuilt, &expanded_directories, &loaded_directories);

        assert_eq!(
            rebuilt_again
                .iter()
                .map(|entry| (entry.path.as_str(), entry.depth))
                .collect::<Vec<_>>(),
            vec![
                ("/root/.openwarp/remote-server", 0),
                ("/root/.openwarp/remote-server/warp-oss", 1),
            ]
        );
    }

    #[test]
    fn selected_index_navigation_stays_in_bounds() {
        assert_eq!(previous_index(None, 0), None);
        assert_eq!(next_index(None, 0), None);
        assert_eq!(previous_index(None, 3), Some(0));
        assert_eq!(previous_index(Some(0), 3), Some(0));
        assert_eq!(previous_index(Some(2), 3), Some(1));
        assert_eq!(next_index(None, 3), Some(1));
        assert_eq!(next_index(Some(1), 3), Some(2));
        assert_eq!(next_index(Some(2), 3), Some(2));
    }

    #[test]
    fn selected_index_preserves_matching_path_after_rebuild() {
        let entries = vec![
            entry("/root/.openwarp", ".openwarp", FileSystemEntryKind::Directory, 0),
            entry(
                "/root/.openwarp/remote-server",
                "remote-server",
                FileSystemEntryKind::Directory,
                1,
            ),
        ];

        assert_eq!(
            selected_index_after_rebuild(&entries, Some("/root/.openwarp/remote-server"), Some(0)),
            Some(1)
        );
    }

    #[test]
    fn selected_index_falls_back_when_collapsed_child_disappears() {
        let entries = vec![entry(
            "/root/.openwarp",
            ".openwarp",
            FileSystemEntryKind::Directory,
            0,
        )];

        assert_eq!(
            selected_index_after_rebuild(
                &entries,
                Some("/root/.openwarp/remote-server"),
                Some(4),
            ),
            Some(0)
        );
    }
}
