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
use warp_core::ui::theme::color::internal_colors;
use warp_core::HostId;
use warp_util::standardized_path::StandardizedPath;
use warpui::clipboard::ClipboardContent;
use warpui::elements::{
    Border, ChildAnchor, ConstrainedBox, Container, CornerRadius, CrossAxisAlignment, Element, Empty,
    Flex, Hoverable, MainAxisSize, MouseStateHandle, OffsetPositioning,
    ParentAnchor, ParentElement, ParentOffsetBounds, Radius, SavePosition, Shrinkable, Stack, Text,
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
use crate::remote_server::manager::RemoteServerManager;
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
const CONTEXT_MENU_WIDTH: f32 = 190.0;
const CONTEXT_MENU_POSITION_ID: &str = "server_file_browser_panel_root";
const TRANSFER_CHUNK_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug)]
pub enum ServerFileBrowserAction {
    Refresh,
    JumpToPath,
    ClickEntry(String),
    ToggleDirectory(String),
    OpenContextMenu {
        path: String,
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
    current_path: String,
    path_editor: ViewHandle<EditorView>,
    entries: Vec<ServerFileBrowserEntry>,
    expanded_directories: HashSet<String>,
    loaded_directories: HashMap<String, Vec<ServerFileBrowserEntry>>,
    selected_path: Option<String>,
    loading: bool,
    status: Option<String>,
    refresh_button: MouseStateHandle,
    upload_file_button: MouseStateHandle,
    upload_folder_button: MouseStateHandle,
    row_states: HashMap<String, MouseStateHandle>,
    context_menu_position: Option<Vector2F>,
    context_menu_target: Option<String>,
    context_menu_item_states: Vec<MouseStateHandle>,
}

impl ServerFileBrowserView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
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
            current_path: String::new(),
            path_editor,
            entries: Vec::new(),
            expanded_directories: HashSet::new(),
            loaded_directories: HashMap::new(),
            selected_path: None,
            loading: false,
            status: Some(crate::t!("server-file-browser-empty")),
            refresh_button: Default::default(),
            upload_file_button: Default::default(),
            upload_folder_button: Default::default(),
            row_states: HashMap::new(),
            context_menu_position: None,
            context_menu_target: None,
            context_menu_item_states: (0..5).map(|_| MouseStateHandle::default()).collect(),
        }
    }

    pub fn set_remote_root(
        &mut self,
        host_id: HostId,
        path: String,
        ctx: &mut ViewContext<Self>,
    ) {
        let should_load = self.host_id.as_ref() != Some(&host_id) || self.current_path != path;
        self.host_id = Some(host_id);
        if should_load {
            self.current_path = path;
            self.sync_editor_to_current_path(ctx);
            self.load_current_directory(ctx);
        }
    }

    pub fn on_left_panel_focused(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.focus(&self.path_editor);
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
        let Some(client) = self.client(ctx) else {
            self.set_error(crate::t!("server-file-browser-no-session"), ctx);
            return;
        };
        let path = if self.current_path.is_empty() {
            "~".to_string()
        } else {
            self.current_path.clone()
        };
        self.loading = true;
        self.status = None;
        ctx.notify();

        ctx.spawn(
            async move { list_directory(client, path).await },
            |me, result, ctx| {
                me.loading = false;
                match result {
                    Ok((canonical_path, entries)) => {
                        me.current_path = canonical_path;
                        me.sync_editor_to_current_path(ctx);
                        me.expanded_directories.clear();
                        me.loaded_directories.clear();
                        me.entries = entries;
                        me.status = None;
                    }
                    Err(error) => {
                        me.status = Some(error);
                    }
                }
                ctx.notify();
            },
        );
    }

    fn resolve_and_open(&mut self, path: String, ctx: &mut ViewContext<Self>) {
        let Some(client) = self.client(ctx) else {
            self.set_error(crate::t!("server-file-browser-no-session"), ctx);
            return;
        };
        let host_id = self.host_id.clone();
        self.loading = true;
        self.status = None;
        ctx.notify();

        ctx.spawn(
            async move { resolve_path(client, path).await },
            move |me, result, ctx| {
                me.loading = false;
                match result {
                    Ok(resolved) if resolved.kind == FileSystemEntryKind::Directory => {
                        me.current_path = resolved.canonical_path;
                        me.sync_editor_to_current_path(ctx);
                        me.load_current_directory(ctx);
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
                            me.current_path = parent;
                            me.sync_editor_to_current_path(ctx);
                            me.load_current_directory(ctx);
                        }
                    }
                    Ok(_) => {
                        me.status = Some(crate::t!("server-file-browser-unsupported-path"));
                    }
                    Err(error) => {
                        me.status = Some(error);
                    }
                }
                ctx.notify();
            },
        );
    }

    fn toggle_directory(&mut self, path: String, ctx: &mut ViewContext<Self>) {
        if self.expanded_directories.remove(&path) {
            self.rebuild_entries();
            ctx.notify();
            return;
        }

        self.expanded_directories.insert(path.clone());
        if self.loaded_directories.contains_key(&path) {
            self.rebuild_entries();
            ctx.notify();
            return;
        }

        let Some(client) = self.client(ctx) else {
            self.set_error(crate::t!("server-file-browser-no-session"), ctx);
            return;
        };
        self.loading = true;
        ctx.notify();
        ctx.spawn(
            async move { list_directory(client, path).await },
            |me, result, ctx| {
                me.loading = false;
                match result {
                    Ok((path, entries)) => {
                        me.loaded_directories.insert(path, entries);
                        me.rebuild_entries();
                    }
                    Err(error) => me.status = Some(error),
                }
                ctx.notify();
            },
        );
    }

    fn rebuild_entries(&mut self) {
        let roots = self.entries.iter().filter(|entry| entry.depth == 0).cloned().collect();
        let mut rebuilt = Vec::new();
        self.append_entries(roots, &mut rebuilt);
        self.entries = rebuilt;
    }

    fn append_entries(
        &self,
        entries: Vec<ServerFileBrowserEntry>,
        out: &mut Vec<ServerFileBrowserEntry>,
    ) {
        for entry in entries {
            let path = entry.path.clone();
            out.push(entry);
            if self.expanded_directories.contains(&path) {
                if let Some(children) = self.loaded_directories.get(&path) {
                    self.append_entries(children.clone(), out);
                }
            }
        }
    }

    fn open_path(&mut self, path: String, ctx: &mut ViewContext<Self>) {
        let Some(entry) = self.entries.iter().find(|entry| entry.path == path).cloned() else {
            return;
        };
        self.selected_path = Some(path.clone());
        match entry.kind {
            FileSystemEntryKind::Directory => self.toggle_directory(path, ctx),
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

    fn open_context_menu(&mut self, path: String, position: Vector2F, ctx: &mut ViewContext<Self>) {
        self.selected_path = Some(path.clone());
        self.context_menu_target = Some(path);
        self.context_menu_position = Some(position);
        ctx.notify();
    }

    fn dismiss_context_menu(&mut self, ctx: &mut ViewContext<Self>) {
        self.context_menu_target = None;
        self.context_menu_position = None;
        ctx.notify();
    }

    fn copy_path(&mut self, path: String, ctx: &mut ViewContext<Self>) {
        ctx.clipboard()
            .write(ClipboardContent::plain_text(path.clone()));
        self.status = Some(crate::t!("server-file-browser-copied-path"));
        self.dismiss_context_menu(ctx);
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
            .with_child(
                ConstrainedBox::new(
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
                .with_width(230.0)
                .finish(),
            )
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
        let mut col = Flex::column();

        if self.host_id.is_none() {
            col.add_child(self.render_status_text(crate::t!("server-file-browser-no-session"), appearance));
        } else if self.loading && self.entries.is_empty() {
            col.add_child(self.render_status_text(crate::t!("server-file-browser-loading"), appearance));
        } else if self.entries.is_empty() {
            col.add_child(self.render_status_text(crate::t!("server-file-browser-empty-directory"), appearance));
        } else {
            for entry in &self.entries {
                col.add_child(self.render_row(entry, appearance));
            }
        }

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
                .with_main_axis_size(MainAxisSize::Min)
                .finish(),
        )
        .with_padding_left(PANEL_HORIZONTAL_PADDING - ITEM_PADDING_HORIZONTAL)
        .with_padding_right(PANEL_HORIZONTAL_PADDING - ITEM_PADDING_HORIZONTAL);

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

    fn render_row(
        &self,
        entry: &ServerFileBrowserEntry,
        appearance: &crate::appearance::Appearance,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let icon_color = theme.sub_text_color(theme.background());
        let is_selected = self.selected_path.as_deref() == Some(entry.path.as_str());
        let is_directory = entry.kind == FileSystemEntryKind::Directory;
        let is_expanded = self.expanded_directories.contains(&entry.path);

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
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_spacing(ITEM_ICON_TEXT_SPACING)
            .with_child(
                ConstrainedBox::new(Empty::new().finish())
                    .with_width(entry.depth as f32 * 16.0)
                    .finish(),
            )
            .with_child(chevron)
            .with_child(icon_el)
            .with_child(text_column)
            .with_main_axis_size(MainAxisSize::Min)
            .finish();

        let state = self.row_states.get(&entry.path).cloned().unwrap_or_default();
        let click_path = entry.path.clone();
        let toggle_path = entry.path.clone();
        let menu_path = entry.path.clone();
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
            ctx.dispatch_typed_action(ServerFileBrowserAction::ClickEntry(click_path.clone()));
        })
        .on_double_click(move |ctx, _, _| {
            ctx.dispatch_typed_action(ServerFileBrowserAction::ToggleDirectory(toggle_path.clone()));
        })
        .on_right_click(move |ctx, _, position| {
            let offset = match ctx.element_position_by_id(CONTEXT_MENU_POSITION_ID) {
                Some(bounds) => position - bounds.origin(),
                None => position,
            };
            ctx.dispatch_typed_action(ServerFileBrowserAction::OpenContextMenu {
                path: menu_path.clone(),
                position: offset,
            });
        })
        .finish();

        Container::new(hoverable).finish()
    }

    fn render_context_menu(&self, appearance: &crate::appearance::Appearance) -> Box<dyn Element> {
        let Some(target) = self.context_menu_target.clone() else {
            return Empty::new().finish();
        };
        let target_entry = self.entries.iter().find(|entry| entry.path == target);
        let target_is_directory = target_entry
            .map(|entry| entry.kind == FileSystemEntryKind::Directory)
            .unwrap_or(false);
        let upload_target = if target_is_directory {
            target.clone()
        } else {
            remote_parent(&target).unwrap_or_else(|| self.current_path.clone())
        };
        let mut items = vec![
            (
                crate::t!("server-file-browser-menu-download"),
                ServerFileBrowserAction::Download(target.clone()),
            ),
            (
                crate::t!("server-file-browser-menu-upload-file"),
                ServerFileBrowserAction::UploadFiles(upload_target.clone()),
            ),
            (
                crate::t!("server-file-browser-menu-upload-folder"),
                ServerFileBrowserAction::UploadFolder(upload_target),
            ),
            (
                crate::t!("server-file-browser-menu-copy-path"),
                ServerFileBrowserAction::CopyPath(target.clone()),
            ),
        ];
        if target_is_directory {
            items.push((
                crate::t!("server-file-browser-menu-jump-to-path"),
                ServerFileBrowserAction::JumpToDirectory(target),
            ));
        }

        let theme = appearance.theme();
        let mut col = Flex::column();
        for (idx, (label, action)) in items.into_iter().enumerate() {
            let state = self
                .context_menu_item_states
                .get(idx)
                .cloned()
                .unwrap_or_default();
            col.add_child(
                Hoverable::new(state, move |_| {
                    Container::new(
                        Text::new_inline(label.clone(), appearance.ui_font_family(), ITEM_FONT_SIZE)
                            .with_color(theme.main_text_color(theme.background()).into())
                            .finish(),
                    )
                    .with_padding_top(7.0)
                    .with_padding_bottom(7.0)
                    .with_padding_left(12.0)
                    .with_padding_right(12.0)
                    .finish()
                })
                .with_cursor(Cursor::PointingHand)
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(action.clone());
                })
                .finish(),
            );
        }

        ConstrainedBox::new(
            Container::new(col.finish())
                .with_background(theme.surface_1())
                .with_corner_radius(CornerRadius::with_all(Radius::Pixels(6.0)))
                .with_border(Border::all(1.0).with_border_fill(theme.nonactive_ui_detail()))
                .finish(),
        )
        .with_width(CONTEXT_MENU_WIDTH)
        .finish()
    }
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
            ServerFileBrowserAction::ClickEntry(path) => self.open_path(path.clone(), ctx),
            ServerFileBrowserAction::ToggleDirectory(path) => self.toggle_directory(path.clone(), ctx),
            ServerFileBrowserAction::OpenContextMenu { path, position } => {
                self.open_context_menu(path.clone(), *position, ctx);
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
            ctx.focus(&self.path_editor);
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

        let Some(position) = self.context_menu_position else {
            return panel;
        };

        let menu = self.render_context_menu(appearance);
        let positioning = OffsetPositioning::offset_from_parent(
            position,
            ParentOffsetBounds::ParentByPosition,
            ParentAnchor::TopLeft,
            ChildAnchor::TopLeft,
        );

        let mut stack = Stack::new();
        stack.add_child(panel);
        stack.add_positioned_overlay_child(menu, positioning);
        stack.finish()
    }
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
