#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use egui::style::{Visuals, WidgetVisuals};
use egui::util::IdTypeMap;
use egui::{Pos2, Response, Ui, UiBuilder};
use parking_lot::{ArcMutexGuard, RawMutex};
use taffy::prelude::*;
use widgets::TaffySeparator;

////////////////////////////////////////////////////////////////////////////////

pub use taffy;

/// Widgets built combining multiple taffy nodes
pub mod widgets;

/// Implemets functionality for egui widgets to be used as taffy leaf containers
mod egui_widgets;

/// Helper functionality for virtual elements
pub mod virtual_tui;

/// Helper function to initialize taffy layout
pub fn tui(ui: &mut egui::Ui, id: impl Into<egui::Id>) -> TuiInitializer<'_> {
    TuiInitializer {
        ui,
        id: id.into(),
        allocated_rect: None,
        available_space: Size {
            width: AvailableSpace::MinContent,
            height: AvailableSpace::MinContent,
        },
        style: Default::default(),
        known_size: Size {
            width: None,
            height: None,
        },
    }
}

/// Egui tui initialization helper to reserve/allocate necessary space
#[must_use]
pub struct TuiInitializer<'a> {
    ui: &'a mut egui::Ui,
    allocated_rect: Option<egui::Rect>,
    available_space: Size<AvailableSpace>,
    known_size: Size<Option<f32>>,
    style: taffy::Style,
    id: egui::Id,
}

impl<'a> TuiInitializer<'a> {
    /// Place ui in already allocated rectangle
    pub fn with_allocated_rect(mut self, rect: egui::Rect) -> TuiInitializer<'a> {
        self.allocated_rect = Some(rect);
        self.available_space = Size {
            width: AvailableSpace::Definite(rect.width()),
            height: AvailableSpace::Definite(rect.height()),
        };
        self
    }

    /// Reserve space
    pub fn reserve_space(self, space: egui::Vec2) -> TuiInitializer<'a> {
        self.reserve_width(space.x).reserve_height(space.y)
    }

    /// Reserve specific width for tui
    pub fn reserve_width(mut self, width: f32) -> TuiInitializer<'a> {
        self.ui.set_min_width(width);
        self.available_space.width = AvailableSpace::Definite(width);
        self.known_size.width = Some(width);
        self
    }

    /// Reserve specific height for tui
    pub fn reserve_height(mut self, height: f32) -> TuiInitializer<'a> {
        self.ui.set_min_height(height);
        self.available_space.height = AvailableSpace::Definite(height);
        self.known_size.height = Some(height);
        self
    }

    /// Reserve all available width
    pub fn reserve_available_space(self) -> TuiInitializer<'a> {
        self.reserve_available_width().reserve_available_height()
    }

    /// Reserve all available width
    pub fn reserve_available_width(self) -> TuiInitializer<'a> {
        let width = self.ui.available_size().x;
        self.reserve_width(width)
    }

    /// Reserve all available height
    pub fn reserve_available_height(self) -> TuiInitializer<'a> {
        let height = self.ui.available_size().y;
        self.reserve_height(height)
    }

    /// Set custom sizing constraints for taffy layouting algorithm for available space
    pub fn with_available_space(
        mut self,
        available_space: Size<AvailableSpace>,
    ) -> TuiInitializer<'a> {
        self.available_space = available_space;
        self
    }

    /// Set root container style
    pub fn style(mut self, style: taffy::Style) -> TuiInitializer<'a> {
        self.style = style;
        self
    }

    /// Show tui
    pub fn show<T>(self, f: impl FnOnce(&mut Tui) -> T) -> T {
        let ui = self.ui;
        let output = Tui::create(
            ui,
            self.id,
            ui.available_rect_before_wrap(),
            Some(self.available_space),
            self.style,
            |tui| {
                // Temporary scroll area size limitation
                tui.set_limit_scroll_area_size(Some(0.7));

                f(tui)
            },
        );

        if self.allocated_rect.is_none() {
            // Space was not allocated yet, allocate used space
            let size = output.container.layout.content_size;
            ui.allocate_space(egui::Vec2 {
                x: size.width,
                y: size.height,
            });
        }
        output.inner
    }
}

/// Tui (Egui Taffy UI) is used to place ui nodes and set their id, style configuration
pub struct Tui {
    main_id: egui::Id,

    ui: egui::Ui,

    current_id: egui::Id,
    current_node: Option<NodeId>,
    current_node_index: usize,
    current_viewport: egui::Rect,
    current_viewport_content: egui::Rect,
    current_rect: egui::Rect,
    taffy_container: TaffyContainerUi,

    last_scroll_offset: egui::Vec2,

    root_rect: egui::Rect,
    available_space: Option<Size<AvailableSpace>>,

    /// Temporary default limit on scroll area size due to taffy
    /// being unable to shrink container to be smaller than content automatically
    limit_scroll_area_size: Option<f32>,

    state: ArcMutexGuard<RawMutex, TaffyState>,

    /// Due to how egui style works with deeply nested structures,
    /// to avoid large amount of [`egui::Style`]` copies
    /// we can cache some style changes
    interactive_container_inactive_style_cache:
        HashMap<(*const egui::Style, InteractiveElementVisualCacheKey), Arc<egui::Style>>,
}

impl Tui {
    /// Manually create Tui, user must manually allocate space in egui Ui if this method is used
    /// directly instead of helper method [`tui`]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(ui, root_rect, available_space, style, f))
    )]
    pub fn create<T>(
        ui: &mut Ui,
        id: egui::Id,
        root_rect: egui::Rect,
        available_space: Option<Size<AvailableSpace>>,
        style: Style,
        f: impl FnOnce(&mut Tui) -> T,
    ) -> TaffyReturn<T> {
        let ui = ui.new_child(UiBuilder::new());

        // Create stored state
        let state = ui.data_mut(|data: &mut IdTypeMap| {
            let state: Arc<parking_lot::Mutex<TaffyState>> = data
                .get_temp_mut_or_insert_with(id, || {
                    Arc::new(parking_lot::Mutex::new(TaffyState::new()))
                })
                .clone();
            state
        });
        let state = state
            .try_lock_arc()
            .expect("Each egui_taffy instance should have unique id");

        let mut this = Self {
            main_id: id,
            ui,
            current_node: None,
            current_node_index: 0,
            current_rect: root_rect,
            current_viewport: root_rect,
            current_viewport_content: root_rect,
            taffy_container: Default::default(),
            root_rect,
            available_space,
            current_id: id,
            limit_scroll_area_size: None,
            last_scroll_offset: egui::Vec2::ZERO,
            state,
            interactive_container_inactive_style_cache: Default::default(),
        };

        let res = this.tui().id(id).style(style).add(|state| {
            let resp = f(state);
            let container = state.recalculate();
            TaffyReturn {
                inner: resp,
                container,
            }
        });

        log::trace!(
            "Cached {} interactive styles!",
            this.interactive_container_inactive_style_cache.len()
        );

        res
    }

    /// Set maximal size coefficient of scroll area based on root element size
    ///
    /// `scroll_area max height = root_height * size`
    ///
    /// Taffy doesn't correctly shrink nodes that should have larger content than their size
    /// (overflow)
    pub fn set_limit_scroll_area_size(&mut self, size: Option<f32>) {
        self.limit_scroll_area_size = size;
    }

    /// Add taffy child node, correctly update taffy tree state
    fn add_child_node(
        &mut self,
        id: egui::Id,
        style: taffy::Style,
        sticky: egui::Vec2b,
    ) -> (NodeId, TaffyContainerUi) {
        let child_idx = self.current_node_index;
        self.current_node_index += 1;

        let mut first_frame = false;

        let state: &mut TaffyState = &mut self.state;

        let node_id = match state.id_to_node_id.entry(id) {
            std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                let val = occupied_entry.get_mut();

                if val.keep {
                    log::error!("Taffy layout id collision!");
                }
                val.keep = true;

                let node_id = val.node_id;
                if state.taffy_tree.style(node_id).unwrap() != &style {
                    state.taffy_tree.set_style(node_id, style).unwrap();
                }
                node_id
            }
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                first_frame = true;
                let node_id = state.taffy_tree.new_leaf(style).unwrap();
                vacant_entry.insert(NodeData {
                    node_id,
                    keep: true,
                });
                node_id
            }
        };

        if let Some(current_node) = self.current_node {
            if child_idx < state.taffy_tree.child_count(current_node) {
                // Check if child at position matches
                if state
                    .taffy_tree
                    .child_at_index(current_node, child_idx)
                    .unwrap()
                    != node_id
                {
                    // Layout has changed, remove all following children
                    //
                    // Because node one by one removal is slow if items have changed their location.
                    // Faster is to remove whole tail.
                    let count = state.taffy_tree.child_count(current_node);
                    state
                        .taffy_tree
                        .remove_children_range(current_node, child_idx..count)
                        .unwrap();

                    // Add element to the end
                    state.taffy_tree.add_child(current_node, node_id).unwrap();
                }
            } else {
                // Add element to the end
                state.taffy_tree.add_child(current_node, node_id).unwrap();
            }
        }

        let container = TaffyContainerUi {
            layout: *state.layout(node_id),
            parent_rect: self.current_rect,
            first_frame,
            sticky,
            last_scroll_offset: self.last_scroll_offset,
        };

        (node_id, container)
    }

    /// Add child taffy node to the layout with optional function to draw background
    #[inline]
    fn add_child<FR, B>(
        &mut self,
        params: TuiBuilderParams,
        background_draw: B,
        f: impl FnOnce(&mut Tui, &mut <B as BackgroundDraw>::ReturnValue) -> FR,
    ) -> TaffyMainBackgroundReturnValues<FR, B::ReturnValue>
    where
        B: BackgroundDraw,
    {
        let mut background_slot = stackbox::Slot::VACANT;
        let mut ui_slot = stackbox::Slot::VACANT;

        self.add_child_dyn(
            params,
            background_slot.stackbox(background_draw).into_dyn(),
            ui_slot.stackbox(f).into_dyn(),
        )
    }

    fn add_child_dyn<FR, BR>(
        &mut self,
        params: TuiBuilderParams,
        background_draw: StackBoxDynBackgroundDrawDyn<BR>,
        f: StackBoxDynFnOnceTuiUi<BR, FR>,
    ) -> TaffyMainBackgroundReturnValues<FR, BR> {
        let TuiBuilderParams {
            id,
            style,
            disabled,
            wrap_mode,
            egui_style,
            layout,
            sticky,
        } = params;

        let style = style.unwrap_or_default();

        let id = id.resolve(self);

        let overflow_style = style.overflow;

        let (node_id, mut current_taffy_container) = self.add_child_node(id, style, sticky);

        let stored_id = self.current_id;
        let stored_node = self.current_node;
        let stored_current_node_index = self.current_node_index;
        let stored_current_rect = self.current_rect;

        std::mem::swap(&mut current_taffy_container, &mut self.taffy_container);
        let stored_taffy_container = current_taffy_container;

        let mut full_container_without_border =
            self.taffy_container.full_container_without_border();
        full_container_without_border = if full_container_without_border.any_nan() {
            self.current_rect
        } else {
            full_container_without_border
        };

        self.current_id = id;
        self.current_node = Some(node_id);
        self.current_node_index = 0;
        self.current_rect = self.taffy_container.full_container();

        let mut ui_builder = egui::UiBuilder::new()
            .id_salt(id.with("_ui"))
            // This does not set clipping, therefore we can still paint outside child ui
            // (on border) and avoid initialising two child user interfaces
            .max_rect(full_container_without_border);

        ui_builder.style = egui_style;
        ui_builder.layout = layout;
        ui_builder.disabled = disabled;

        let mut child_ui = self.ui.new_child(ui_builder);
        child_ui.expand_to_include_rect(full_container_without_border);

        if let Some(wrap_mode) = wrap_mode {
            if child_ui.style().wrap_mode != Some(wrap_mode) {
                child_ui.style_mut().wrap_mode = Some(wrap_mode);
            }
        }

        let mut bg = match background_draw.simulate_execution_dyn() {
            Some(val) => val,
            None => background_draw.draw_dyn(&mut child_ui, &self.taffy_container),
        };

        let fg = {
            let mut scroll_in_directions = egui::Vec2b::FALSE;
            match overflow_style.y {
                taffy::Overflow::Visible => {
                    // Do nothing
                }
                taffy::Overflow::Clip | taffy::Overflow::Hidden | taffy::Overflow::Scroll => {
                    // Add scroll area
                    if overflow_style.y == taffy::Overflow::Scroll {
                        scroll_in_directions.y = true;
                    }
                    // Hide overflow
                    let mut clip_rect = child_ui.clip_rect();
                    clip_rect.min.y = full_container_without_border.min.y;
                    clip_rect.max.y = full_container_without_border.max.y;
                    child_ui.shrink_clip_rect(clip_rect);
                }
            }

            match overflow_style.y {
                taffy::Overflow::Visible => {
                    // Do nothing
                }
                taffy::Overflow::Clip | taffy::Overflow::Hidden | taffy::Overflow::Scroll => {
                    // Add scroll area
                    if overflow_style.x == taffy::Overflow::Scroll {
                        scroll_in_directions.x = true;
                    }

                    // Hide overflow
                    let mut clip_rect = child_ui.clip_rect();
                    clip_rect.min.x = full_container_without_border.min.x;
                    clip_rect.max.x = full_container_without_border.max.x;
                    child_ui.shrink_clip_rect(clip_rect);
                }
            }

            if scroll_in_directions.any() {
                let scroll = egui::ScrollArea::new(scroll_in_directions)
                    .min_scrolled_width(full_container_without_border.width())
                    .max_width(full_container_without_border.width())
                    .min_scrolled_height(full_container_without_border.height())
                    .max_height(full_container_without_border.height())
                    .show(&mut child_ui, |ui| {
                        // Allocate expected size for scroll area to correctly calculate inner size
                        let content_size = self.taffy_container.layout.content_size;
                        ui.set_min_size(
                            egui::Vec2::new(content_size.width, content_size.height)
                                .max(egui::Vec2::ZERO),
                        );

                        let mut rect = ui.min_rect();
                        let mut offset = rect.min - self.current_rect.min;

                        let stored_viewport = self.current_viewport;
                        let stored_viewport_content = self.current_viewport_content;

                        self.current_viewport = self.current_rect;
                        self.current_viewport_content = rect;
                        std::mem::swap(&mut self.last_scroll_offset, &mut offset);
                        std::mem::swap(&mut self.current_rect, &mut rect);
                        std::mem::swap(ui, &mut self.ui);

                        let resp = f.show_dyn(self, &mut bg);

                        std::mem::swap(ui, &mut self.ui);
                        std::mem::swap(&mut self.current_rect, &mut rect);
                        std::mem::swap(&mut self.last_scroll_offset, &mut offset);
                        self.current_viewport_content = stored_viewport_content;
                        self.current_viewport = stored_viewport;

                        resp
                    });
                scroll.inner
            } else {
                std::mem::swap(&mut child_ui, &mut self.ui);
                let resp = f.show_dyn(self, &mut bg);
                std::mem::swap(&mut child_ui, &mut self.ui);
                resp
            }
        };

        let current_cnt = self.state.taffy_tree.child_count(node_id);
        if current_cnt > self.current_node_index {
            self.state
                .taffy_tree
                .remove_children_range(node_id, self.current_node_index..current_cnt)
                .unwrap();
        }

        self.current_id = stored_id;
        self.current_node = stored_node;
        self.current_node_index = stored_current_node_index;
        self.current_rect = stored_current_rect;
        self.taffy_container = stored_taffy_container;

        TaffyMainBackgroundReturnValues {
            main: fg,
            background: bg,
        }
    }

    #[inline]
    fn add_container<T>(
        &mut self,
        params: TuiBuilderParams,
        content: impl FnOnce(&mut Ui, &TaffyContainerUi) -> TuiContainerResponse<T>,
    ) -> T {
        let mut ui_slot = stackbox::Slot::VACANT;
        self.add_container_dyn(params, ui_slot.stackbox(content).into_dyn())
    }

    /// Add egui user interface as child node in the Tui
    fn add_container_dyn<T>(
        &mut self,
        params: TuiBuilderParams,
        content: StackBoxDynFnOnceEguiUiContainer<T>,
    ) -> T {
        let fg_bg = self.add_child(params, (), |tui, _| {
            let taffy_container = &tui.taffy_container;

            let mut ui_builder = UiBuilder::new()
                .max_rect(taffy_container.full_container_without_border_and_padding());
            if taffy_container.first_frame {
                ui_builder = ui_builder.sizing_pass().invisible();
            }
            let mut child_ui = tui.ui.new_child(ui_builder);

            let resp = content.show_dyn(&mut child_ui, taffy_container);

            let nodeid = tui.current_node.unwrap();

            let min_size = if let Some(intrinsic_size) = resp.intrinsic_size {
                resp.min_size.min(intrinsic_size).ceil()
            } else {
                resp.min_size.ceil()
            };

            let mut max_size = resp.max_size;
            max_size = max_size.max(min_size);

            let new_content = Context {
                min_size,
                max_size,
                infinite: resp.infinite,
            };
            if tui.state.taffy_tree.get_node_context(nodeid) != Some(&new_content) {
                tui.state
                    .taffy_tree
                    .set_node_context(nodeid, Some(new_content))
                    .unwrap();
            }

            resp.inner
        });

        fg_bg.main
    }

    /// Add scroll area egui Ui to the taffy layout
    fn ui_scroll_area_ext<T>(
        &mut self,
        mut params: TuiBuilderParams,
        limit: Option<f32>,
        content: impl FnOnce(&mut Ui) -> T,
    ) -> T {
        let style = params.style.get_or_insert_with(Style::default);

        style.overflow = taffy::Point {
            x: taffy::Overflow::Visible,
            y: taffy::Overflow::Hidden,
        };
        style.display = taffy::Display::Block;
        style.min_size = Size {
            width: Dimension::Length(0.),
            height: Dimension::Length(0.),
        };
        if let Some(limit) = limit {
            style.max_size.height = Dimension::Length(self.root_rect.height() * limit);
            style.max_size.width = Dimension::Length(self.root_rect.width() * limit);
        }

        self.tui().params(params).add(|tui| {
            let layout = *tui
                .state
                .taffy_tree
                .layout(tui.current_node.unwrap())
                .unwrap();

            tui.add_container(
                TuiBuilderParams {
                    id: "inner".into(),
                    style: None,
                    disabled: false,
                    wrap_mode: None,
                    egui_style: None,
                    layout: None,
                    sticky: egui::Vec2b::FALSE,
                },
                |ui, _params| {
                    let mut real_min_size = None;
                    let scroll_area = egui::ScrollArea::both()
                        .id_salt(ui.id().with("scroll_area"))
                        .max_width(ui.available_width())
                        .min_scrolled_width(layout.size.width)
                        .max_width(layout.size.width)
                        .min_scrolled_height(layout.size.height)
                        .max_height(layout.size.height)
                        .show(ui, |ui| {
                            let resp = content(ui);
                            real_min_size = Some(ui.min_size());
                            resp
                        });

                    let potential_frame_size = scroll_area.content_size;

                    // let min_size = egui::Vec2 {
                    //     x: potential_frame_size.x,
                    //     y: 0.3 * potential_frame_size.y,
                    // };
                    let max_size = egui::Vec2 {
                        x: potential_frame_size.x,
                        y: potential_frame_size.y,
                    };

                    TuiContainerResponse {
                        inner: scroll_area.inner,
                        min_size: real_min_size.unwrap_or(max_size),
                        intrinsic_size: None,
                        max_size,
                        infinite: egui::Vec2b::FALSE,
                    }
                },
            )
        })
    }

    /// Check if tui layout has changed, recalculate if necessary and trigger
    /// request discard for egui to redraw the UI
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn recalculate(&mut self) -> TaffyContainerUi {
        let root_rect = self.root_rect;
        let available_space = self.available_space.unwrap_or(Size {
            width: AvailableSpace::Definite(root_rect.width()),
            height: AvailableSpace::Definite(root_rect.height()),
        });

        let current_node = self.current_node.unwrap();

        // Remove all unused nodes
        let state = self.state.deref_mut();
        state.id_to_node_id.retain(|_k, node_data| {
            if node_data.keep {
                node_data.keep = false;
                return true;
            }

            // Remove unused node
            let node_id = node_data.node_id;
            if let Some(parent) = state.taffy_tree.parent(node_id) {
                state.taffy_tree.remove_child(parent, node_id).unwrap();
            }
            state.taffy_tree.remove(node_id).unwrap();
            false
        });

        let taffy = &mut state.taffy_tree;

        if taffy.dirty(current_node).unwrap() || state.last_size != root_rect.size() {
            // let ctx = self.ui.ctx();

            state.last_size = root_rect.size();
            taffy
                .compute_layout_with_measure(
                    current_node,
                    available_space,
                    |_known_size: Size<Option<f32>>,
                     available_space: Size<AvailableSpace>,
                     _id,
                     context,
                     _style|
                     -> Size<f32> {
                        let context = context.copied().unwrap_or(Context {
                            min_size: egui::Vec2::ZERO,
                            max_size: egui::Vec2::ZERO,
                            infinite: egui::Vec2b::FALSE,
                        });

                        let Context {
                            mut min_size,
                            mut max_size,
                            infinite,
                        } = context;

                        // if scroll_area {
                        //     min_size = egui::Vec2::ZERO;
                        // }

                        if min_size.any_nan() {
                            min_size = egui::Vec2::ZERO;
                        }
                        if max_size.any_nan() {
                            max_size = root_rect.size();
                        }

                        let max_size = egui::Vec2 {
                            x: if infinite.x {
                                root_rect.width()
                            } else {
                                max_size.x
                            },
                            y: if infinite.y {
                                root_rect.height()
                            } else {
                                max_size.y
                            },
                        };

                        let width = match available_space.width {
                            AvailableSpace::Definite(num) => {
                                num.clamp(min_size.x, max_size.x.max(min_size.x))
                            }
                            AvailableSpace::MinContent => min_size.x,
                            AvailableSpace::MaxContent => max_size.x,
                        };
                        let height = match available_space.height {
                            AvailableSpace::Definite(num) => {
                                num.clamp(min_size.y, max_size.y.max(min_size.y))
                            }
                            AvailableSpace::MinContent => min_size.y,
                            AvailableSpace::MaxContent => max_size.y,
                        };

                        #[allow(clippy::let_and_return)]
                        let final_size = Size { width, height };

                        // println!(
                        //     "{:?} {:?} {:?} {:?} {:?} {:?}",
                        //     _id, min_size, max_size, available_space, final_size, _known_size,
                        // );

                        final_size
                    },
                )
                .unwrap();
            // taffy.print_tree(current_node);

            log::trace!("Taffy recalculation done!");
            self.ui.ctx().request_discard("Taffy recalculation");
        }

        TaffyContainerUi {
            parent_rect: root_rect,
            layout: *self.state.layout(current_node),
            first_frame: false,
            sticky: egui::Vec2b::FALSE,
            last_scroll_offset: egui::Vec2::ZERO,
        }
    }

    /// Access underlaying egui ui
    #[inline]
    pub fn egui_ui(&self) -> &egui::Ui {
        &self.ui
    }

    /// Access underlaying egui ui
    #[inline]
    pub fn egui_ctx(&self) -> &egui::Context {
        self.ui.ctx()
    }

    /// Access underlaying egui ui
    #[inline]
    pub fn egui_ui_mut(&mut self) -> &mut egui::Ui {
        &mut self.ui
    }

    /// Calling `disable()` will cause the [`egui::Ui`] to deny all future interaction
    /// and all the widgets will draw with a gray look.
    ///
    /// Note that once disabled, there is no way to re-enable the [`egui::Ui`].
    ///
    /// Shorthand for
    /// ```ignore
    /// tui.egui_ui_mut().disable();
    /// ```
    #[inline]
    pub fn disable(&mut self) {
        self.egui_ui_mut().disable();
    }

    /// Modify underlaying egui style
    #[inline]
    pub fn egui_style_mut(&mut self) -> &mut egui::Style {
        self.ui.style_mut()
    }

    /// Initial root rect size set by the user
    ///
    /// (Used size in reality could change based on available space settings )
    #[inline]
    pub fn root_rect(&self) -> egui::Rect {
        self.root_rect
    }

    /// Retrieve and clone current taffy style
    ///
    /// Useful when need to create child nodes with the same style
    #[inline]
    pub fn current_style(&self) -> &taffy::Style {
        self.state
            .taffy_tree
            .style(self.current_node.unwrap())
            .unwrap()
    }

    /// Current Tui UI id
    #[inline]
    pub fn current_id(&self) -> egui::Id {
        self.current_id
    }

    /// Last viewport rect (Full tui layout or last scrollable element)
    #[inline]
    pub fn current_viewport(&self) -> egui::Rect {
        self.current_viewport
    }

    /// Current viewport content rect (Full tui layout or last scrollable element content rect)
    #[inline]
    pub fn current_viewport_content(&self) -> egui::Rect {
        self.current_viewport_content
    }

    /// Retrieve current Tui node [`NodeId`]
    #[inline]
    pub fn current_node(&self) -> NodeId {
        // Public function is only called when current_node is initialised
        self.current_node.unwrap()
    }

    /// Retrieve layout information of current Tui node
    #[inline]
    pub fn taffy_container(&self) -> &TaffyContainerUi {
        &self.taffy_container
    }

    /// Retrieve inner state of taffy layout
    #[inline]
    fn taffy_state(&self) -> &TaffyState {
        &self.state
    }

    /// Retrieve taffy id that was used to identify this egui_taffy instance in egui data
    #[inline]
    pub fn main_taffy_id(&self) -> egui::Id {
        self.main_id
    }
}

/// Tui returned information about final layout of the Tui
///
/// Can be used to allocate necessary space in parent egui::Ui
pub struct TaffyReturn<T> {
    /// Value returned by closure
    pub inner: T,
    /// Container layout information
    pub container: TaffyContainerUi,
}

/// Sizing context retrieved from Tui layout leaf nodes (egui widgets or child egui::Ui)
///
/// Used to calculate final layout in taffy layout calculations
#[derive(PartialEq, Default, Clone, Copy)]
pub struct Context {
    min_size: egui::Vec2,
    max_size: egui::Vec2,
    infinite: egui::Vec2b,
}

/// Helper to show the inner content of a container.
#[derive(Clone)]
pub struct TaffyContainerUi {
    layout: taffy::Layout,
    parent_rect: egui::Rect,
    last_scroll_offset: egui::Vec2,
    sticky: egui::Vec2b,
    first_frame: bool,
}

impl Default for TaffyContainerUi {
    fn default() -> Self {
        Self {
            layout: Default::default(),
            parent_rect: egui::Rect::ZERO,
            last_scroll_offset: Default::default(),
            sticky: Default::default(),
            first_frame: Default::default(),
        }
    }
}

#[inline]
fn sum_axis(rect: &taffy::Rect<f32>) -> taffy::Size<f32> {
    taffy::Size {
        width: rect.left + rect.right,
        height: rect.top + rect.bottom,
    }
}

#[inline]
fn top_left(rect: &taffy::Rect<f32>) -> taffy::Point<f32> {
    taffy::Point {
        x: rect.left,
        y: rect.top,
    }
}

impl TaffyContainerUi {
    /// Sticky element compensation amount based on last scrollable ancestor scroll offset
    #[inline]
    pub fn sticky_offset(&self) -> egui::Vec2 {
        self.sticky.to_vec2() * self.last_scroll_offset
    }

    /// Full container size
    #[inline]
    pub fn full_container(&self) -> egui::Rect {
        self.full_container_with(true)
    }

    /// Full container size
    #[inline]
    pub fn full_container_with(&self, scroll_offset: bool) -> egui::Rect {
        let layout = &self.layout;
        let rect = egui::Rect::from_min_size(
            Pos2::new(layout.location.x, layout.location.y),
            egui::Vec2::new(layout.size.width, layout.size.height),
        );
        let mut offset = self.parent_rect.min.to_vec2();
        if scroll_offset {
            offset += -self.sticky_offset();
        }
        rect.translate(offset)
    }

    /// Full container rect without border
    #[inline]
    pub fn full_container_without_border(&self) -> egui::Rect {
        let layout = &self.layout;

        let pos = layout.location + top_left(&layout.border);
        let size = layout.size - sum_axis(&layout.border);

        let rect = egui::Rect::from_min_size(
            Pos2::new(pos.x, pos.y),
            egui::Vec2::new(size.width, size.height),
        );
        rect.translate(self.parent_rect.min.to_vec2() - self.sticky_offset())
    }

    /// Full container rect without border and padding
    #[inline]
    pub fn full_container_without_border_and_padding(&self) -> egui::Rect {
        let layout = &self.layout;

        let pos = layout.location + top_left(&layout.padding) + top_left(&layout.border);
        let size = layout.size - sum_axis(&layout.padding) - sum_axis(&layout.border);

        let rect = egui::Rect::from_min_size(
            Pos2::new(pos.x, pos.y),
            egui::Vec2::new(size.width, size.height),
        );
        rect.translate(self.parent_rect.min.to_vec2() - self.sticky_offset())
    }

    /// Calculated taffy::Layout for this node
    #[inline]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Is this the first frame.
    #[inline]
    pub fn first_frame(&self) -> bool {
        self.first_frame
    }

    /// Parent rect that is used to calculate rect of this node
    #[inline]
    pub fn parent_rect(&self) -> egui::Rect {
        self.parent_rect
    }

    /// Is element position sticky in specified dimensions
    #[inline]
    pub fn sticky(&self) -> egui::Vec2b {
        self.sticky
    }
}

/// Describes information about used space when laying out elements
///
/// This information is used for taffy layout calculation logic
pub struct TuiContainerResponse<T> {
    /// Closure return value
    pub inner: T,
    /// Minimal size that this widget can shrink to
    pub min_size: egui::Vec2,
    /// Minimal size reported by egui widget that this widget can shrink to
    pub intrinsic_size: Option<egui::Vec2>,
    /// Maximal size this widget can grow to
    ///
    /// See [`TuiContainerResponse::infinite`] to support containers that can grow infinitely
    pub max_size: egui::Vec2,
    /// Can widget grow to infinite size in given dimensions
    pub infinite: egui::Vec2b,
}

/// Return values from Main, Background closures
pub struct TaffyMainBackgroundReturnValues<F, B> {
    /// Value returned by main layout function
    pub main: F,
    /// Value returned by background drawing function
    pub background: B,
}

////////////////////////////////////////////////////////////////////////////////

/// Implement this trait for a widget to make it usable in a tui container.
///
/// The reason there is a separate trait is that we need to measure the content size independently
/// of the frame size. (The content will stay at it's intrinsic size while the frame will be
/// stretched according to the flex layout.)
///
/// If your widget has no frame, you can use [`TuiBuilderLogic::ui``] directly to implement this trait.
///
/// See [`crate::widgets`] and `./egui_widgets.rs` for example trait implementations.
///
/// Trait idea taken from egui_flex
pub trait TuiWidget {
    /// The response type of the widget
    type Response;
    /// Show your widget here. Use the provided [`TuiBuilder`] to draw your widget correctly
    /// using the given style.
    fn taffy_ui(self, tuib: TuiBuilder) -> Self::Response;
}

////////////////////////////////////////////////////////////////////////////////

/// Id type to simplify defining layout node ids
#[derive(Default, Clone)]
pub enum TuiId {
    /// Create id based on parent node id and given id
    ///
    /// This is useful to avoid duplicated ids if the same layout is used in multiple places
    Hiarchy(egui::Id),

    /// Unique id, useful if node will be moved around in the layout and
    /// full layout recalculation is not needed
    Unique(egui::Id),

    /// Auto generate id using which child in the sequence this element is in parent node
    #[default]
    Auto,
}

impl TuiId {
    /// Calculate final id based on the current Tui state
    fn resolve(self, tui: &Tui) -> egui::Id {
        match self {
            TuiId::Hiarchy(id) => tui.current_id.with(id),
            TuiId::Unique(id) => id,
            TuiId::Auto => tui.current_id.with("auto").with(tui.current_node_index),
        }
    }
}

impl From<egui::Id> for TuiId {
    #[inline]
    fn from(value: egui::Id) -> Self {
        Self::Hiarchy(value)
    }
}
impl From<&str> for TuiId {
    #[inline]
    fn from(value: &str) -> Self {
        Self::Hiarchy(egui::Id::new(value))
    }
}

/// Helper function to generate TuiID that takes into account element hiarchy to avoid duplicated
/// ids
#[inline]
pub fn tid<T>(id: T) -> TuiId
where
    T: std::hash::Hash,
{
    TuiId::Hiarchy(egui::Id::new(id))
}

////////////////////////////////////////////////////////////////////////////////

/// Egui taffy layout state which stores calculated taffy node layout and hiarchy
pub struct TaffyState {
    taffy_tree: TaffyTree<Context>,

    id_to_node_id: HashMap<egui::Id, NodeData>,

    last_size: egui::Vec2,
}

/// Stores information about node that was identified by egui::Id
pub struct NodeData {
    /// [`taffy::TaffyTree`] node id
    pub node_id: NodeId,
    keep: bool,
}

impl TaffyState {
    fn new() -> Self {
        Self {
            taffy_tree: TaffyTree::new(),
            last_size: egui::Vec2::ZERO,
            id_to_node_id: HashMap::default(),
        }
    }

    #[inline]
    fn layout(&self, node_id: NodeId) -> &Layout {
        self.taffy_tree.layout(node_id).unwrap()
    }

    /// Retrieve underlaying [`TaffyTree`] that stores calculated layout information
    #[inline]
    pub fn taffy_tree(&self) -> &TaffyTree<Context> {
        &self.taffy_tree
    }

    /// Mapping to retrieve nodes based on [`egui::Id`]
    #[inline]
    pub fn items(&self) -> &HashMap<egui::Id, NodeData> {
        &self.id_to_node_id
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Helper structure to provide more egonomic API for child ui container creation
#[must_use]
pub struct TuiBuilder<'r> {
    tui: &'r mut Tui,
    params: TuiBuilderParams,
}

/// Parameters for creating child element in Tui layout
#[derive(Clone)]
pub struct TuiBuilderParams {
    /// Child ui identifier to correctly match elements between frames
    pub id: TuiId,

    /// Child element taffy layout settings / style
    pub style: Option<taffy::Style>,

    /// Should layout descendant egui ui be disabled upon creation
    pub disabled: bool,

    /// Setting to set child ui style wrap_mode
    pub wrap_mode: Option<egui::TextWrapMode>,

    /// Egui style for child ui
    pub egui_style: Option<Arc<egui::Style>>,

    /// Layout for egui child ui
    pub layout: Option<egui::Layout>,

    /// Sticky position (Should last scroll offset affect the position of the element)
    pub sticky: egui::Vec2b,
}

impl<'r> TuiBuilder<'r> {
    /// Retrieve underlaying Tui used to initialize child element
    pub fn builder_tui(&self) -> &&'r mut Tui {
        &self.tui
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Helper trait to reduce code boilerplate
pub trait AsTuiBuilder<'r>: Sized {
    /// Initialize creation of tui new child node
    fn tui(self) -> TuiBuilder<'r>;
}

impl<'r> AsTuiBuilder<'r> for &'r mut Tui {
    #[inline]
    fn tui(self) -> TuiBuilder<'r> {
        TuiBuilder {
            tui: self,
            params: TuiBuilderParams {
                id: TuiId::Auto,
                style: None,
                disabled: false,
                wrap_mode: None,
                egui_style: None,
                layout: None,
                sticky: egui::Vec2b::FALSE,
            },
        }
    }
}

impl<'r> AsTuiBuilder<'r> for TuiBuilder<'r> {
    #[inline]
    fn tui(self) -> TuiBuilder<'r> {
        self
    }
}

impl<'r, T> TuiBuilderLogic<'r> for T
where
    T: AsTuiBuilder<'r>,
{
    // Use default implementation
}

/// Getter function to get values defined by [`egui::style::Visuals`] and [`egui::style::WidgetVisuals`]
type VisualsGetterFn<T> = Box<dyn Fn(&Visuals, &WidgetVisuals) -> T>;
/// Getter function to get values defined by [`egui::style::Visuals`] and [`egui::style::WidgetVisuals`]
/// It includes a reference to [`egui::Response`] to handle widget states
type VisualsResponseGetterFn<T> = Box<dyn Fn(&Visuals, &WidgetVisuals, &Response) -> T>;

/// Generic structure of values to draw a background by [`TuiBackground`]
enum TuiBackgroundValue<T> {
    /// Custom value
    Custom(T),
    /// Value defined by [`egui::style::Visuals`] accessible by a getter function
    Visuals(VisualsGetterFn<T>),
    /// Value depending on [`egui::Visuals`] accessible by a getter function
    VisualsResponse(VisualsResponseGetterFn<T>),
}

struct TuiBackgroundBorder {
    color: TuiBackgroundValue<egui::Color32>,
    width: TuiBackgroundValue<f32>,
}

impl Default for TuiBackgroundBorder {
    fn default() -> Self {
        Self {
            color: TuiBackgroundValue::Visuals(Box::new(|_, widget_visuals| {
                widget_visuals.bg_stroke.color
            })),
            width: TuiBackgroundValue::Visuals(Box::new(|_, widget_visuals| {
                widget_visuals.bg_stroke.width
            })),
        }
    }
}

/// Helper to draw background fills and borders
pub struct TuiBackground {
    background_color: TuiBackgroundValue<egui::Color32>,
    corner_radius: TuiBackgroundValue<egui::CornerRadius>,
    border: Option<TuiBackgroundBorder>,
}

impl Default for TuiBackground {
    fn default() -> Self {
        Self {
            background_color: TuiBackgroundValue::Visuals(Box::new(|visuals, _| visuals.panel_fill)),
            corner_radius: TuiBackgroundValue::Visuals(Box::new(|_, widget_visuals| {
                widget_visuals.corner_radius
            })),
            border: None,
        }
    }
}

impl TuiBackground {
    /// Creates a new instance drawing a default background only
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Draws background by given [`egui::Color32`]
    pub fn with_background_color(self, background_color: egui::Color32) -> Self {
        Self {
            background_color: TuiBackgroundValue::Custom(background_color),
            ..self
        }
    }

    /// Draws background color defined by [`egui::style::Visuals`]
    pub fn with_background_color_by_visuals(self, f: VisualsGetterFn<egui::Color32>) -> Self {
        Self {
            background_color: TuiBackgroundValue::Visuals(f),
            ..self
        }
    }

    /// Draws background color depending on [`VisualsResponseGetterFn`]
    pub fn with_background_color_by_response(
        self,
        f: VisualsResponseGetterFn<egui::Color32>,
    ) -> Self {
        Self {
            background_color: TuiBackgroundValue::VisualsResponse(f),
            ..self
        }
    }

    /// Draws default border
    pub fn with_border(self) -> Self {
        Self {
            border: Some(TuiBackgroundBorder::default()),
            ..self
        }
    }

    /// Draws a border by given [`egui::Color32`]
    pub fn with_border_color(self, color: egui::Color32) -> Self {
        Self {
            border: Some(TuiBackgroundBorder {
                color: TuiBackgroundValue::Custom(color),
                width: self
                    .border
                    .map(|b| b.width)
                    .unwrap_or(TuiBackgroundBorder::default().width),
            }),
            ..self
        }
    }

    /// Draws a border color defined by [`VisualsGetterFn`]
    pub fn with_border_color_by_visuals(self, f: VisualsGetterFn<egui::Color32>) -> Self {
        Self {
            border: Some(TuiBackgroundBorder {
                color: TuiBackgroundValue::Visuals(f),
                width: self
                    .border
                    .map(|b| b.width)
                    .unwrap_or(TuiBackgroundBorder::default().width),
            }),
            ..self
        }
    }

    /// Draws a border color defined by [`VisualsResponseGetterFn`]
    pub fn with_border_color_by_response(self, f: VisualsResponseGetterFn<egui::Color32>) -> Self {
        Self {
            border: Some(TuiBackgroundBorder {
                color: TuiBackgroundValue::VisualsResponse(f),
                width: self
                    .border
                    .map(|b| b.width)
                    .unwrap_or(TuiBackgroundBorder::default().width),
            }),
            ..self
        }
    }

    /// Draws border with given width
    pub fn with_border_width(self, width: f32) -> Self {
        Self {
            border: Some(TuiBackgroundBorder {
                width: TuiBackgroundValue::Custom(width),
                color: self
                    .border
                    .map(|b| b.color)
                    .unwrap_or_else(|| TuiBackgroundBorder::default().color),
            }),
            ..self
        }
    }

    /// Draws a border width defined by [`VisualsGetterFn`]
    pub fn with_border_width_by_visuals(self, f: VisualsGetterFn<f32>) -> Self {
        Self {
            border: Some(TuiBackgroundBorder {
                width: TuiBackgroundValue::Visuals(f),
                color: self
                    .border
                    .map(|b| b.color)
                    .unwrap_or_else(|| TuiBackgroundBorder::default().color),
            }),
            ..self
        }
    }

    /// Draws a border width defined by [`VisualsResponseGetterFn`]
    pub fn with_border_width_by_response(self, f: VisualsResponseGetterFn<f32>) -> Self {
        Self {
            border: Some(TuiBackgroundBorder {
                width: TuiBackgroundValue::VisualsResponse(f),
                color: self
                    .border
                    .map(|b| b.color)
                    .unwrap_or_else(|| TuiBackgroundBorder::default().color),
            }),
            ..self
        }
    }

    /// Draws corner radius with given radius
    pub fn with_corner_radius(self, radius: impl Into<egui::CornerRadius>) -> Self {
        Self {
            corner_radius: TuiBackgroundValue::Custom(radius.into()),
            ..self
        }
    }

    /// Draws background with given [`egui::CornerRadius`] defined by [`VisualsGetterFn`]
    pub fn with_corner_radius_by_visuals(self, f: VisualsGetterFn<egui::CornerRadius>) -> Self {
        Self {
            corner_radius: TuiBackgroundValue::Visuals(f),
            ..self
        }
    }

    /// Draws background with given [`egui::CornerRadius`] defined by [`VisualsResponseGetterFn`]
    pub fn with_corner_radius_by_response(
        self,
        f: VisualsResponseGetterFn<egui::CornerRadius>,
    ) -> Self {
        Self {
            corner_radius: TuiBackgroundValue::VisualsResponse(f),
            ..self
        }
    }

    fn has_border(&self) -> bool {
        self.border.is_some()
    }

    // Internal function to draw content of background.
    fn draw_internal(
        &self,
        ui: &egui::Ui,
        container: &TaffyContainerUi,
        widget_visuals: &egui::style::WidgetVisuals,
        response: Option<&Response>,
    ) {
        let rect = container.full_container();

        let visuals = ui.style().visuals.clone();

        // Helper to get value out from `TuiBackgroundValue`
        fn match_value<T: Clone>(
            visuals: &egui::style::Visuals,
            widget_visuals: &egui::style::WidgetVisuals,
            response: Option<&Response>,
            value: &TuiBackgroundValue<T>,
        ) -> T {
            match value {
                TuiBackgroundValue::Custom(value) => value.clone(),
                TuiBackgroundValue::Visuals(f) => f(visuals, widget_visuals),
                TuiBackgroundValue::VisualsResponse(f) => match response {
                    Some(r) => f(visuals, widget_visuals, r),
                    None => unreachable!("never called without a response"),
                },
            }
        }
        // optional fill
        let fill = match_value(&visuals, widget_visuals, response, &self.background_color);

        // optional stroke
        let stroke = self.border.as_ref().map(|border| {
            let color = match_value(&visuals, widget_visuals, response, &border.color);
            let width = match_value(&visuals, widget_visuals, response, &border.width);
            egui::Stroke { color, width }
        });

        let corner_radius = match_value(&visuals, widget_visuals, response, &self.corner_radius);

        match stroke {
            // border + fill
            Some(stroke) => {
                ui.painter()
                    .rect(rect, corner_radius, fill, stroke, egui::StrokeKind::Inside);
            }
            // fill only
            None => {
                ui.painter().rect_filled(rect, corner_radius, fill);
            }
        }
    }

    /// Returns a draw function can be used by [`BackgroundDraw`]
    pub fn draw(&self) -> impl FnOnce(&mut egui::Ui, &TaffyContainerUi) {
        move |ui: &mut egui::Ui, container: &TaffyContainerUi| {
            let widget_visuals = ui.style().visuals.noninteractive();
            self.draw_internal(ui, container, widget_visuals, None);
        }
    }

    /// Returns a draw function with an [`egui::Response`]
    pub fn draw_with_response(&self) -> impl FnOnce(&mut egui::Ui, &TaffyContainerUi) -> Response {
        move |ui: &mut egui::Ui, container: &TaffyContainerUi| {
            let rect = container.full_container();
            let response = ui.interact(rect, ui.id().with("bg"), egui::Sense::click());
            let widget_visuals = ui.style().interact(&response);
            self.draw_internal(ui, container, widget_visuals, Some(&response));
            response
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Trait that implements TuiBuilder logic for child node creation in Tui UI.
pub trait TuiBuilderLogic<'r>: AsTuiBuilder<'r> + Sized {
    /// Override all child element parameters with given values
    #[inline]
    fn params(self, params: TuiBuilderParams) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params = params;
        tui
    }

    /// Set child node id
    #[inline]
    fn id(self, id: impl Into<TuiId>) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params.id = id.into();
        tui
    }

    /// Set child node style
    #[inline]
    fn style(self, style: taffy::Style) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params.style = Some(style);
        tui
    }

    /// Set child node style to be the same as current node style
    fn reuse_style(self) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params.style = Some(tui.tui.current_style().clone());
        tui
    }

    /// Set child node id and style
    #[inline]
    fn id_style(self, id: impl Into<TuiId>, style: taffy::Style) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params.id = id.into();
        tui.params.style = Some(style);
        tui
    }

    /// Mutate child node style
    #[inline]
    fn mut_style(self, f: impl FnOnce(&mut taffy::Style)) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        f(tui.params.style.get_or_insert_with(Default::default));
        tui
    }

    /// Set child enabled_ui egui flag
    #[inline]
    fn enabled_ui(self, enabled_ui: bool) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params.disabled |= !enabled_ui;
        tui
    }

    /// Set child element to contain disabled [`egui::Ui`]
    ///
    /// See [`egui::Ui::disable`] for more information.
    #[inline]
    fn disabled(self) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params.disabled = true;
        tui
    }

    /// Set child element wrap mode
    #[inline]
    fn wrap_mode(self, wrap_mode: egui::TextWrapMode) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params.wrap_mode = Some(wrap_mode);
        tui
    }

    /// Set child element egui style
    #[inline]
    fn egui_style(self, style: Arc<egui::Style>) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params.egui_style = Some(style);
        tui
    }

    /// Mutate child element egui style
    #[inline]
    fn mut_egui_style(self, f: impl FnOnce(&mut egui::Style)) -> TuiBuilder<'r> {
        let mut tui = self.tui();

        // Unpack style efficiently
        let mut style = if let Some(style) = tui.params.egui_style {
            Arc::unwrap_or_clone(style)
        } else {
            tui.builder_tui().egui_ui().style().deref().clone()
        };
        f(&mut style);

        tui.params.egui_style = Some(Arc::new(style));
        tui
    }

    /// Set child element egui layout
    #[inline]
    fn egui_layout(self, layout: egui::Layout) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params.layout = Some(layout);
        tui
    }

    /// Set element as sticky in specified dimensions.
    ///
    /// Element position in specified dimensions will not be affected by ancestore `overflow: scroll` element
    /// scroll offset in specified dimension.
    #[inline]
    fn sticky(self, sticky: egui::Vec2b) -> TuiBuilder<'r> {
        let mut tui = self.tui();
        tui.params.sticky = sticky;
        tui
    }

    /// Add tui node as children to this node
    #[inline]
    fn add<T>(self, f: impl FnOnce(&mut Tui) -> T) -> T {
        let tui = self.tui();
        tui.tui.add_child(tui.params, (), |tui, _| f(tui)).main
    }

    /// Add empty tui node as children to this node
    ///
    /// Useful to fill grid cells with empty content
    #[inline]
    fn add_empty(self) {
        self.tui().add(|_| {})
    }

    /// Add tui node as children to this node and draw a background
    #[inline]
    fn add_with_background<T>(self, f: impl FnOnce(&mut Tui) -> T) -> T {
        let tui = self.tui();
        let return_values =
            tui.add_with_background_ui(TuiBackground::new().draw(), |tui, _| f(tui));
        return_values.main
    }

    /// Add tui node as children to this node and draw a background with custom color
    #[inline]
    fn add_with_background_color<T>(
        self,
        background_color: egui::Color32,
        f: impl FnOnce(&mut Tui) -> T,
    ) -> T {
        let tui = self.tui();
        let return_values = tui.add_with_background_ui(
            TuiBackground::new()
                .with_background_color(background_color)
                .draw(),
            |tui, _| f(tui),
        );
        return_values.main
    }

    /// Add tui node as children to this node and draw a bordered background
    #[inline]
    fn add_with_background_bordered<T>(self, f: impl FnOnce(&mut Tui) -> T) -> T {
        let tui = self.tui().with_border_style_from_egui_style();
        let return_values =
            tui.add_with_background_ui(TuiBackground::new().with_border().draw(), |tui, _| f(tui));
        return_values.main
    }

    /// To correctly layout element with border,
    /// set taffy style border size parameter
    /// from egui noninteractive widget visual bg_stroke width.
    fn with_border_style_from_egui_style(self) -> TuiBuilder<'r> {
        let tui = self.tui();
        let border = tui.tui.egui_ui().style().noninteractive().bg_stroke.width;
        tui.mut_style(|style| {
            // Allocate space for border in layout
            if style.border == Rect::zero() {
                style.border = length(border);
            }
        })
    }

    /// Add tui node as children to this node and draw simple group Frame background
    #[inline]
    fn add_with_border<T>(self, f: impl FnOnce(&mut Tui) -> T) -> T {
        let return_values = self
            .with_border_style_from_egui_style()
            .add_with_background_ui(TuiBackground::new().with_border().draw(), |tui, _| f(tui));
        return_values.main
    }

    /// Add tui node as children to this node and draw a custom background and/or a custom border
    /// defined by given [`TuiBackground`]
    ///
    /// Use this function if you want to have full control of [`TuiBackground`] values
    #[inline]
    fn add_with_background_ext<T>(
        self,
        background: TuiBackground,
        f: impl FnOnce(&mut Tui) -> T,
    ) -> T {
        let tui = self.tui();
        tui.tui
            .add_child(tui.params, background.draw(), |tui, _| f(tui))
            .main
    }

    /// Add tui node with a custom background that is clickable
    ///
    /// Use this function if you want to have full control of [`TuiBackground`] values
    #[must_use = "You should check if the user clicked this with `if ….clicked() { … } "]
    #[inline]
    fn clickable_with_background_ext<T>(
        self,
        background: TuiBackground,
        f: impl FnOnce(&mut Tui) -> T,
    ) -> TuiInnerResponse<T> {
        let tui = if background.has_border() {
            self.with_border_style_from_egui_style()
        } else {
            self.tui()
        };

        let return_values = tui.tui.add_child(
            tui.params,
            background.draw_with_response(),
            |tui, bg_response| {
                setup_tui_visuals(tui, bg_response);
                f(tui)
            },
        );

        TuiInnerResponse {
            inner: return_values.main,
            response: return_values.background,
        }
    }

    /// Add tui node with a transparent background that is clickable
    /// Use `clickable_ext` to style background and / or borders
    #[must_use = "You should check if the user clicked this with `if ….clicked() { … } "]
    fn clickable<T>(self, f: impl FnOnce(&mut Tui) -> T) -> TuiInnerResponse<T> {
        self.clickable_with_background_ext(
            TuiBackground::new().with_background_color(egui::Color32::TRANSPARENT),
            f,
        )
    }

    /// Add tui node with background that acts as egui button
    #[must_use = "You should check if the user clicked this with `if ….clicked() { … } "]
    #[inline]
    fn filled_button<T>(
        self,
        target_tint_color: Option<egui::Color32>,
        f: impl FnOnce(&mut Tui) -> T,
    ) -> TuiInnerResponse<T> {
        let background_color: VisualsGetterFn<egui::Color32> = match target_tint_color {
            None => {
                Box::new(|_: &Visuals, widget_visuals: &WidgetVisuals| widget_visuals.weak_bg_fill)
            }
            Some(tint_color) => Box::new(move |_: &Visuals, widget_visuals: &WidgetVisuals| {
                egui::ecolor::tint_color_towards(widget_visuals.weak_bg_fill, tint_color)
            }),
        };

        let background = TuiBackground::new()
            .with_border()
            .with_background_color_by_visuals(background_color);

        self.clickable_with_background_ext(background, f)
    }

    /// Add tui node with background that acts as egui button
    #[must_use = "You should check if the user clicked this with `if ….clicked() { … } "]
    #[inline]
    fn button<T>(self, f: impl FnOnce(&mut Tui) -> T) -> TuiInnerResponse<T> {
        self.filled_button(None, f)
    }

    /// Add tui node with background that acts as selectable button
    #[must_use = "You should check if the user clicked this with `if ….clicked() { … } "]
    #[inline]
    fn selectable<T>(self, selected: bool, f: impl FnOnce(&mut Tui) -> T) -> TuiInnerResponse<T> {
        let background_color = Box::new(
            move |visuals: &Visuals, widget_visuals: &WidgetVisuals, response: &Response| {
                if response.hovered() && selected {
                    // Add visual effect even if button is selected
                    visuals.gray_out(widget_visuals.weak_bg_fill)
                } else {
                    widget_visuals.weak_bg_fill
                }
            },
        );

        let background = TuiBackground::new()
            .with_border()
            .with_background_color_by_response(background_color);

        self.clickable_with_background_ext(background, f)
    }

    /// Add tui node as children to this node and draw custom background
    ///
    /// See [`TuiBuilderLogic::add_with_background`] for example
    #[inline]
    fn add_with_background_ui<FR, BR>(
        self,
        content: impl FnOnce(&mut egui::Ui, &TaffyContainerUi) -> BR,
        f: impl FnOnce(&mut Tui, &mut BR) -> FR,
    ) -> TaffyMainBackgroundReturnValues<FR, BR> {
        let tui = self.tui();
        tui.tui.add_child(tui.params, content, f)
    }

    /// Add scroll area egui Ui
    ///
    /// Alternative: Using `overflow: Scroll` scroll area will be directly inserted in taffy layout.
    fn ui_scroll_area_with_background<T>(self, content: impl FnOnce(&mut Ui) -> T) -> T {
        let mut tui = self.tui();
        tui = tui.mut_style(|style| {
            if style.min_size.height == Dimension::Auto {
                style.min_size.height = Dimension::Length(0.);
            }
            if style.min_size.width == Dimension::Auto {
                style.min_size.width = Dimension::Length(0.);
            }
        });

        tui.add_with_background_bordered(move |tui| {
            let s = LengthPercentageAuto::Length(
                0.3 * tui.ui.text_style_height(&egui::TextStyle::Body),
            );
            let style = taffy::Style {
                margin: Rect {
                    left: s,
                    right: s,
                    top: s,
                    bottom: s,
                },
                ..Default::default()
            };
            tui.tui().style(style).ui_scroll_area(content)
        })
    }

    /// Add scroll area egui Ui
    ///
    /// Alternative: Using `overflow: Scroll` scroll area will be directly inserted in taffy layout.
    fn ui_scroll_area<T>(self, content: impl FnOnce(&mut Ui) -> T) -> T {
        let tui = self.tui();
        let limit = tui.tui.limit_scroll_area_size;
        tui.ui_scroll_area_ext(limit, content)
    }

    /// Add egui::Ui scroll area with custom limit for scroll area size
    ///
    /// Alternative: Using `overflow: Scroll` scroll area will be directly inserted in taffy layout.
    fn ui_scroll_area_ext<T>(self, limit: Option<f32>, content: impl FnOnce(&mut Ui) -> T) -> T {
        let tui = self.tui();
        tui.tui.ui_scroll_area_ext(tui.params, limit, content)
    }

    /// Add egui ui as tui leaf node
    #[inline]
    fn ui<T>(self, content: impl FnOnce(&mut Ui) -> T) -> T {
        self.ui_finite(content)
    }

    /// Add finite egui ui as tui leaf node
    #[inline]
    fn ui_finite<T>(self, content: impl FnOnce(&mut Ui) -> T) -> T {
        self.ui_manual(|ui, _params| {
            let inner = content(ui);
            TuiContainerResponse {
                inner,
                min_size: ui.min_size(),
                intrinsic_size: None,
                max_size: ui.min_size(),
                infinite: egui::Vec2b::FALSE,
            }
        })
    }

    /// Add egui ui that can grow infinitely as tui leaf node
    #[inline]
    fn ui_infinite<T>(self, content: impl FnOnce(&mut Ui) -> T) -> T {
        self.ui_manual(|ui, _params| {
            let inner = content(ui);
            TuiContainerResponse {
                inner,
                min_size: ui.min_size(),
                intrinsic_size: None,
                max_size: ui.min_size(),
                infinite: egui::Vec2b::TRUE,
            }
        })
    }

    /// Add egui ui as tui leaf node and provide custom information about necessary space for this
    /// node for layout calculation
    ///
    /// Useful when implementing [`TuiWidget`] trait
    #[inline]
    fn ui_manual<T>(
        self,
        content: impl FnOnce(&mut Ui, &TaffyContainerUi) -> TuiContainerResponse<T>,
    ) -> T {
        let tui = self.tui();
        tui.tui.add_container(tui.params, content)
    }

    /// Add tui or egui widget that implements [`TuiWidget`]` as leaf node
    #[inline]
    fn ui_add<T: TuiWidget>(self, widget: T) -> T::Response {
        widget.taffy_ui(self.tui())
    }

    /// Add egui widget as leaf node and modify calculated used space information
    ///
    /// Useful when implementing [`TuiWidget`] trait
    #[inline]
    fn ui_add_manual(
        self,
        f: impl FnOnce(&mut egui::Ui) -> Response,
        transform: impl FnOnce(
            TuiContainerResponse<Response>,
            &egui::Ui,
        ) -> TuiContainerResponse<Response>,
    ) -> Response {
        self.ui_manual(|ui, _params| {
            let response = f(ui);

            let resp = TuiContainerResponse {
                min_size: response.rect.size(),
                intrinsic_size: response.intrinsic_size,
                max_size: response.rect.size(),
                infinite: egui::Vec2b::FALSE,
                inner: response,
            };

            transform(resp, ui)
        })
    }

    /// Add egui label as child node
    #[inline]
    fn label(self, text: impl Into<egui::WidgetText>) -> Response {
        egui::Label::new(text).taffy_ui(self.tui())
    }

    /// Add egui colored label as child node
    #[inline]
    fn colored_label(self, color: egui::Color32, text: impl Into<egui::RichText>) -> Response {
        egui::Label::new(text.into().color(color)).taffy_ui(self.tui())
    }

    /// Add label as child node with strong visual formatting
    #[inline]
    fn strong(self, text: impl Into<egui::RichText>) -> Response {
        egui::Label::new(text.into().strong()).taffy_ui(self.tui())
    }

    /// Add egui heading as child node
    #[inline]
    fn heading(self, text: impl Into<egui::RichText>) -> Response {
        egui::Label::new(text.into().heading()).taffy_ui(self.tui())
    }

    /// Add egui small text as child node
    #[inline]
    fn small(self, text: impl Into<egui::RichText>) -> Response {
        egui::Label::new(text.into().small()).taffy_ui(self.tui())
    }

    /// Add egui separator  as child node
    ///
    /// Seperator is drawn perpendiculary to parent element flex_direction (main_axis)
    #[inline]
    fn separator(self) -> Response {
        TaffySeparator::default().taffy_ui(self.tui())
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Helper structure to return:
///     1. [`egui::Response`] of the surrounding element,
///     2. return value of the inner closure.
#[derive(Debug)]
pub struct TuiInnerResponse<R> {
    /// What the user closure returned.
    pub inner: R,

    /// The response of the area.
    pub response: egui::Response,
}

impl<R> std::ops::Deref for TuiInnerResponse<R> {
    type Target = egui::Response;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Types that can draw background
///
/// [`()`] type draws empty background.
trait BackgroundDraw {
    /// Value returned by background drawing functionality
    type ReturnValue;

    /// Function returns Some(value) if background doesn't need to be drawn
    fn simulate_execution(&self) -> Option<Self::ReturnValue>;

    /// Implements background drawing functionality
    fn draw(self, ui: &mut egui::Ui, container: &TaffyContainerUi) -> Self::ReturnValue;
}

impl<T, B> BackgroundDraw for T
where
    T: FnOnce(&mut egui::Ui, &TaffyContainerUi) -> B,
{
    type ReturnValue = B;

    #[inline]
    fn draw(self, ui: &mut egui::Ui, container: &TaffyContainerUi) -> Self::ReturnValue {
        self(ui, container)
    }

    #[inline]
    fn simulate_execution(&self) -> Option<Self::ReturnValue> {
        None
    }
}

impl BackgroundDraw for () {
    type ReturnValue = ();

    #[inline]
    fn draw(self, ui: &mut egui::Ui, container: &TaffyContainerUi) -> Self::ReturnValue {
        let _ = container;
        let _ = ui;
    }

    #[inline]
    fn simulate_execution(&self) -> Option<Self::ReturnValue> {
        Some(())
    }
}

stackbox::custom_dyn! {
    dyn BackgroundDrawDyn<Ret> : BackgroundDraw<ReturnValue = Ret>
    {
        fn draw_dyn(
            self: Self,
            ui: &mut egui::Ui,
            container: &TaffyContainerUi,
        ) -> Ret {
            self.draw(ui, container)
        }

        fn simulate_execution_dyn(self: &Self) -> Option<Ret> {
            self.simulate_execution()
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Front ui rendering logic
trait FrontUi<Context, Ret> {
    fn show(self, tui: &mut Tui, bret: &mut Context) -> Ret;
}

impl<T, Context, Ret> FrontUi<Context, Ret> for T
where
    T: FnOnce(&mut Tui, &mut Context) -> Ret,
{
    #[inline]
    fn show(self, tui: &mut Tui, bret: &mut Context) -> Ret {
        self(tui, bret)
    }
}

stackbox::custom_dyn! {
    dyn FnOnceTuiUi<Context, Ret> : FrontUi<Context, Ret>
    {
        fn show_dyn(self: Self, tui: &mut Tui, bret: &mut Context) -> Ret {
            self.show(tui, bret)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Egui node display trait
trait EguiUiContainer<Ret> {
    fn show(self, ui: &mut Ui, container: &TaffyContainerUi) -> TuiContainerResponse<Ret>;
}

impl<T, Ret> EguiUiContainer<Ret> for T
where
    T: FnOnce(&mut egui::Ui, &TaffyContainerUi) -> TuiContainerResponse<Ret>,
{
    #[inline]
    fn show(self, ui: &mut egui::Ui, container: &TaffyContainerUi) -> TuiContainerResponse<Ret> {
        self(ui, container)
    }
}

stackbox::custom_dyn! {
    dyn FnOnceEguiUiContainer<Ret> : EguiUiContainer<Ret>
    {
        fn show_dyn(self: Self, ui: &mut egui::Ui, container: &TaffyContainerUi) -> TuiContainerResponse<Ret> {
            self.show(ui, container)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Hash, PartialEq, Eq)]
enum InteractiveElementVisualCacheKey {
    Inactive,
    Active,
    Hovered,
}

/// Helper function to set up tui visuals based on background response interaction state
pub fn setup_tui_visuals(tui: &mut Tui, bg_response: &Response) {
    let response = bg_response;
    let style = tui.ui.style();
    let visuals = &style.visuals.widgets;

    // See `[egui::style::Visuals]`
    let (cache_key, visuals) = if !response.sense.interactive() {
        // Nothing to change, fast exit to avoid unnecessary copies of egui::Style
        (
            InteractiveElementVisualCacheKey::Inactive,
            &visuals.inactive,
        )
    } else if response.is_pointer_button_down_on() || response.has_focus() || response.clicked() {
        (InteractiveElementVisualCacheKey::Active, &visuals.active)
    } else if response.hovered() || response.highlighted() {
        (InteractiveElementVisualCacheKey::Hovered, &visuals.hovered)
    } else {
        // Nothing to change, fast exit to avoid unnecessary copies of egui::Style
        (
            InteractiveElementVisualCacheKey::Inactive,
            &visuals.inactive,
        )
    };

    // WARN: Optimization to avoid egui::Style full cloning on every interactive element
    let cached_style = tui
        .interactive_container_inactive_style_cache
        .entry((Arc::as_ptr(style), cache_key))
        .or_insert_with(|| {
            let mut egui_style: egui::Style = style.deref().clone();
            egui_style.interaction.selectable_labels = false;
            egui_style.visuals.widgets.inactive = *visuals;
            egui_style.visuals.widgets.noninteractive = *visuals;
            Arc::new(egui_style)
        })
        .clone();
    tui.egui_ui_mut().set_style(cached_style);
}
