#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use egui::util::IdTypeMap;
use egui::{Pos2, Response, Ui, UiBuilder};
use taffy::prelude::*;
use widgets::TaffySeparator;

////////////////////////////////////////////////////////////////////////////////

pub use taffy;

/// Widgets built combining multiple taffy nodes
pub mod widgets;

/// Implemets functionality for egui widgets to be used as taffy leaf containers
mod egui_widgets;

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
    last_child_count: usize,

    parent_rect: egui::Rect,
    last_scroll_offset: egui::Vec2,

    used_items: HashSet<egui::Id>,

    root_rect: egui::Rect,
    available_space: Option<Size<AvailableSpace>>,

    /// Temporary default limit on scroll area size due to taffy
    /// being unable to shrink container to be smaller than content automatically
    limit_scroll_area_size: Option<f32>,
}

impl Tui {
    /// Retrieve stored layout information in egui memory
    fn with_state<T>(id: egui::Id, ctx: egui::Context, f: impl FnOnce(&mut TaffyState) -> T) -> T {
        let state = ctx.data_mut(|data: &mut IdTypeMap| {
            let state: Arc<Mutex<TaffyState>> = data
                .get_temp_mut_or_insert_with(id, || Arc::new(Mutex::new(TaffyState::new())))
                .clone();
            state
        });

        let mut state = state.lock().unwrap();

        f(&mut state)
    }

    /// Manually create Tui, user must manually allocate space in egui Ui if this method is used
    /// directly instead of helper method [`tui`]
    pub fn create<T>(
        ui: &mut Ui,
        id: egui::Id,
        root_rect: egui::Rect,
        available_space: Option<Size<AvailableSpace>>,
        style: Style,
        f: impl FnOnce(&mut Tui) -> T,
    ) -> TaffyReturn<T> {
        let ui = ui.new_child(UiBuilder::new());

        let mut this = Self {
            main_id: id,
            ui,
            current_node: None,
            current_node_index: 0,
            last_child_count: 0,
            parent_rect: root_rect,
            used_items: Default::default(),
            root_rect,
            available_space,
            current_id: id,
            limit_scroll_area_size: None,
            last_scroll_offset: egui::Vec2::ZERO,
        };

        this.tui().id(id).style(style).add(|state| {
            let resp = f(state);
            let container = state.recalculate();
            TaffyReturn {
                inner: resp,
                container,
            }
        })
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
        if self.used_items.contains(&id) {
            log::error!("Taffy layout id collision!");
        }

        Self::with_state(self.main_id, self.ui.ctx().clone(), |state| {
            let child_idx = self.current_node_index;
            self.current_node_index += 1;

            self.used_items.insert(id);
            let mut first_frame = false;

            let node_id = if let Some(node_id) = state.items.get(&id).copied() {
                if state.taffy.style(node_id).unwrap() != &style {
                    state.taffy.set_style(node_id, style).unwrap();
                }
                node_id
            } else {
                first_frame = true;
                let node = state.taffy.new_leaf(style).unwrap();
                state.items.insert(id, node);
                node
            };

            if let Some(current_node) = self.current_node {
                if child_idx < self.last_child_count {
                    if state.taffy.child_at_index(current_node, child_idx).unwrap() != node_id {
                        state
                            .taffy
                            .replace_child_at_index(current_node, child_idx, node_id)
                            .unwrap();
                    }
                } else {
                    state.taffy.add_child(current_node, node_id).unwrap();
                    self.last_child_count += 1;
                }
            }

            let container = TaffyContainerUi {
                layout: state.layout(node_id),
                parent_rect: self.parent_rect,
                first_frame,
                sticky,
                last_scroll_offset: self.last_scroll_offset,
            };

            (node_id, container)
        })
    }

    /// Add child taffy node to the layout with optional function to draw background
    fn add_children_inner<T>(
        &mut self,
        params: TuiBuilderParams,
        content: Option<impl FnOnce(&mut egui::Ui, &TaffyContainerUi)>,
        f: impl FnOnce(&mut Tui, TaffyContainerUi) -> T,
    ) -> T {
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

        let (node_id, taffy_container) = self.add_child_node(id, style, sticky);

        let stored_id = self.current_id;
        let stored_node = self.current_node;
        let stored_node_index = self.current_node_index;
        let stored_last_child_count = self.last_child_count;
        let stored_parent_rect = self.parent_rect;

        Self::with_state(self.main_id, self.ui.ctx().clone(), |state| {
            self.current_node = Some(node_id);
            self.current_node_index = 0;
            self.last_child_count = state.taffy.child_count(node_id);

            let max_rect = taffy_container.full_container();
            self.parent_rect = if max_rect.any_nan() {
                self.parent_rect
            } else {
                max_rect
            };
            self.current_id = id;
        });

        let full_container_max_rect = taffy_container.full_container();

        if let Some(content) = content {
            if !full_container_max_rect.any_nan() {
                let mut ui_builder = egui::UiBuilder::new()
                    .id_salt(id.with("background"))
                    .max_rect(full_container_max_rect);

                ui_builder.style = egui_style.clone();
                ui_builder.layout = layout;

                let mut child_ui = self.ui.new_child(ui_builder);

                if let Some(wrap_mode) = wrap_mode {
                    if child_ui.style().wrap_mode != Some(wrap_mode) {
                        child_ui.style_mut().wrap_mode = Some(wrap_mode);
                    }
                }

                if disabled {
                    // Can not set this in UiBuilder,
                    // because it does not correctly set up color fade out for
                    // disabled elements
                    child_ui.disable();
                }

                content(&mut child_ui, &taffy_container);
            }
        }

        let resp = {
            let full_container_without_border = taffy_container.full_container_without_border();
            let mut ui_builder = egui::UiBuilder::new()
                .id_salt(id)
                .max_rect(full_container_without_border);
            ui_builder.style = egui_style;
            ui_builder.layout = layout;

            let mut tmp_ui = self.ui.new_child(ui_builder);

            if disabled {
                // Can not set this in UiBuilder,
                // because it does not correctly set up color fade out for
                // disabled elements
                tmp_ui.disable();
            }

            if let Some(wrap_mode) = wrap_mode {
                if tmp_ui.style().wrap_mode != Some(wrap_mode) {
                    tmp_ui.style_mut().wrap_mode = Some(wrap_mode);
                }
            }

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
                    let mut clip_rect = tmp_ui.clip_rect();
                    clip_rect.min.y = full_container_without_border.min.y;
                    clip_rect.max.y = full_container_without_border.max.y;
                    tmp_ui.shrink_clip_rect(clip_rect);
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
                    let mut clip_rect = tmp_ui.clip_rect();
                    clip_rect.min.x = full_container_without_border.min.x;
                    clip_rect.max.x = full_container_without_border.max.x;
                    tmp_ui.shrink_clip_rect(clip_rect);
                }
            }

            if scroll_in_directions.any() {
                let scroll = egui::ScrollArea::new(scroll_in_directions)
                    .min_scrolled_width(full_container_without_border.width())
                    .max_width(full_container_without_border.width())
                    .min_scrolled_height(full_container_without_border.height())
                    .max_height(full_container_without_border.height())
                    .show(&mut tmp_ui, |ui| {
                        // Allocate expected size for scroll area to correctly calculate inner size
                        let content_size = taffy_container.layout.content_size;
                        let (mut rect, _resp) = ui.allocate_exact_size(
                            // TODO: Fix -1 workaround
                            // -1 due to unknown bug in calculation and redundant scrollbar
                            // Maybe egui rounds something. See scrollbar demo.
                            egui::Vec2::new(content_size.width - 1., content_size.height - 1.)
                                .max(egui::Vec2::ZERO),
                            egui::Sense::hover(),
                        );

                        let mut offset = rect.min - self.parent_rect.min;
                        std::mem::swap(&mut self.last_scroll_offset, &mut offset);
                        std::mem::swap(&mut self.parent_rect, &mut rect);
                        std::mem::swap(ui, &mut self.ui);
                        let resp = f(self, taffy_container);
                        std::mem::swap(ui, &mut self.ui);
                        std::mem::swap(&mut self.parent_rect, &mut rect);
                        std::mem::swap(&mut self.last_scroll_offset, &mut offset);
                        resp
                    });
                scroll.inner
            } else {
                std::mem::swap(&mut tmp_ui, &mut self.ui);
                let resp = f(self, taffy_container);
                std::mem::swap(&mut tmp_ui, &mut self.ui);
                resp
            }
        };

        Self::with_state(self.main_id, self.ui.ctx().clone(), |state| {
            let mut current_cnt = state.taffy.child_count(node_id);

            while current_cnt > self.last_child_count {
                state
                    .taffy
                    .remove_child_at_index(node_id, current_cnt - 1)
                    .unwrap();
                current_cnt -= 1;
            }
        });

        self.current_id = stored_id;
        self.current_node = stored_node;
        self.current_node_index = stored_node_index;
        self.last_child_count = stored_last_child_count;
        self.parent_rect = stored_parent_rect;

        resp
    }

    /// Add egui user interface as child node in the Tui
    fn add_container<T>(
        &mut self,
        params: TuiBuilderParams,
        content: impl FnOnce(&mut Ui, TaffyContainerUi) -> TuiContainerResponse<T>,
    ) -> T {
        self.add_children_inner(
            params,
            None::<fn(&mut egui::Ui, &TaffyContainerUi)>,
            |tui, taffy_container| {
                let mut ui_builder = UiBuilder::new()
                    .max_rect(taffy_container.full_container_without_border_and_padding());
                if taffy_container.first_frame {
                    ui_builder = ui_builder.sizing_pass().invisible();
                }
                let mut child_ui = tui.ui.new_child(ui_builder);

                let resp = content(&mut child_ui, taffy_container);

                let nodeid = tui.current_node.unwrap();

                Self::with_state(tui.main_id, tui.ui.ctx().clone(), |state| {
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
                    if state.taffy.get_node_context(nodeid) != Some(&new_content) {
                        state
                            .taffy
                            .set_node_context(nodeid, Some(new_content))
                            .unwrap();
                    }
                });

                resp.inner
            },
        )
    }

    /// Add scroll area egui Ui to the taffy layout
    fn ui_scroll_area_ext<T>(
        &mut self,
        mut params: TuiBuilderParams,
        limit: Option<f32>,
        content: impl FnOnce(&mut Ui) -> T,
    ) -> T {
        let style = params.style.get_or_insert_with(|| Style::default());

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
            let layout = Self::with_state(tui.main_id, tui.ui.ctx().clone(), |state| {
                *state.taffy.layout(tui.current_node.unwrap()).unwrap()
            });

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
    fn recalculate(&mut self) -> TaffyContainerUi {
        let root_rect = self.root_rect;
        let available_space = self.available_space.unwrap_or(Size {
            width: AvailableSpace::Definite(root_rect.width()),
            height: AvailableSpace::Definite(root_rect.height()),
        });

        let current_node = self.current_node.unwrap();
        Self::with_state(self.main_id, self.ui.ctx().clone(), |state| {
            // Remove unused nodes (Removes unused child nodes too )
            state.items.retain(|k, v| {
                if self.used_items.contains(k) {
                    return true;
                }
                if let Some(parent) = state.taffy.parent(*v) {
                    state.taffy.remove_child(parent, *v).unwrap();
                }
                state.taffy.remove(*v).unwrap();
                false
            });
            self.used_items.clear();

            let taffy = &mut state.taffy;

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
                layout: state.layout(current_node),
                first_frame: false,
                sticky: egui::Vec2b::FALSE,
                last_scroll_offset: egui::Vec2::ZERO,
            }
        })
    }

    /// Access underlaying egui ui
    #[inline]
    pub fn egui_ui(&self) -> &egui::Ui {
        &self.ui
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
    pub fn root_rect(&self) -> egui::Rect {
        self.root_rect
    }

    /// Retrieve and clone current taffy style
    ///
    /// Useful when need to create child nodes with the same style
    pub fn current_style(&self) -> taffy::Style {
        Self::with_state(self.main_id, self.ui.ctx().clone(), |data| {
            data.taffy
                .style(self.current_node.unwrap())
                .unwrap()
                .clone()
        })
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
struct Context {
    min_size: egui::Vec2,
    max_size: egui::Vec2,
    infinite: egui::Vec2b,
}

/// Helper to show the inner content of a container.
pub struct TaffyContainerUi {
    layout: taffy::Layout,
    parent_rect: egui::Rect,
    last_scroll_offset: egui::Vec2,
    sticky: egui::Vec2b,
    first_frame: bool,
}

fn sum_axis(rect: &taffy::Rect<f32>) -> taffy::Size<f32> {
    taffy::Size {
        width: rect.left + rect.right,
        height: rect.top + rect.bottom,
    }
}

fn top_left(rect: &taffy::Rect<f32>) -> taffy::Point<f32> {
    taffy::Point {
        x: rect.left,
        y: rect.top,
    }
}

impl TaffyContainerUi {
    /// Sticky element compensation amount based on last scrollable ancestor scroll offset
    pub fn sticky_offset(&self) -> egui::Vec2 {
        self.sticky.to_vec2() * self.last_scroll_offset
    }

    /// Full container size
    pub fn full_container(&self) -> egui::Rect {
        let layout = &self.layout;
        let rect = egui::Rect::from_min_size(
            Pos2::new(layout.location.x, layout.location.y),
            egui::Vec2::new(layout.size.width, layout.size.height),
        );
        rect.translate(self.parent_rect.min.to_vec2() - self.sticky_offset())
    }

    /// Full container rect without border
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
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Is this the first frame.
    pub fn first_frame(&self) -> bool {
        self.first_frame
    }

    /// Parent rect that is used to calculate rect of this node
    pub fn parent_rect(&self) -> egui::Rect {
        self.parent_rect
    }

    /// Is element position sticky in specified dimensions
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

/// Taffy layout state that stores calculated taffy node layout and hiarchy
struct TaffyState {
    taffy: TaffyTree<Context>,

    last_size: egui::Vec2,
    items: HashMap<egui::Id, NodeId>,
}

impl TaffyState {
    fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            last_size: egui::Vec2::ZERO,
            items: HashMap::default(),
        }
    }

    fn layout(&self, node_id: NodeId) -> Layout {
        *self.taffy.layout(node_id).unwrap()
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
        tui.params.style = Some(tui.tui.current_style());
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
    fn add<T>(self, f: impl FnOnce(&mut Tui) -> T) -> T {
        let tui = self.tui();
        tui.tui.add_children_inner(
            tui.params,
            Option::<fn(&mut egui::Ui, &TaffyContainerUi)>::None,
            |tui, _| f(tui),
        )
    }

    /// Add empty tui node as children to this node
    ///
    /// Useful to fill grid cells with empty content
    fn add_empty(self) {
        self.tui().add(|_| {})
    }

    /// Add tui node as children to this node and draw popup background
    fn add_with_background<T>(self, f: impl FnOnce(&mut Tui) -> T) -> T {
        let tui = self.tui();
        let border = tui.tui.egui_ui().style().noninteractive().bg_stroke.width;
        let tui = tui.mut_style(|style| {
            // Allocate space for border in layout
            if style.border == Rect::zero() {
                style.border = length(border);
            }
        });
        tui.add_with_background_ui(
            |ui, container| {
                let available_space = ui.available_size();
                let (id, rect) = ui.allocate_space(available_space);
                let _response = ui.interact(rect, id, egui::Sense::click_and_drag());
                // Background is not transparent to events

                let visuals = ui.style().visuals.noninteractive();
                let window_fill = ui.style().visuals.panel_fill;

                let rect = container.full_container();

                let painter = ui.painter();
                painter.rect_filled(rect, visuals.rounding, window_fill);
                painter.rect_stroke(rect, visuals.rounding, visuals.bg_stroke);
            },
            f,
        )
    }

    /// To correctly layout element with border,
    /// set taffy style border size parameter
    /// from egui noninteractive widget visual bg_stroke width.
    fn with_border_style_from_egui_style(self) -> TuiBuilder<'r> {
        let tui = self.tui();
        let border = tui.tui.egui_ui().style().noninteractive().bg_stroke.width;
        let tui = tui.mut_style(|style| {
            // Allocate space for border in layout
            if style.border == Rect::zero() {
                style.border = length(border);
            }
        });
        tui
    }

    /// Add tui node as children to this node and draw simple group Frame background
    fn add_with_border<T>(self, f: impl FnOnce(&mut Tui) -> T) -> T {
        self.with_border_style_from_egui_style()
            .add_with_background_ui(
                |ui, container| {
                    let noninteractive = ui.style().noninteractive();
                    let max_rect = container.full_container();

                    // Background is transparent to events
                    ui.painter().rect_stroke(
                        max_rect,
                        noninteractive.rounding,
                        noninteractive.bg_stroke,
                    );
                },
                f,
            )
    }

    /// Add tui node with background that acts as egui button
    #[must_use = "You should check if the user clicked this with `if ….clicked() { … } "]
    fn button<T>(self, f: impl FnOnce(&mut Tui) -> T) -> TuiInnerResponse<T> {
        let tui = self.with_border_style_from_egui_style();
        let data =
            std::cell::RefCell::<Option<(egui::style::WidgetVisuals, egui::Response)>>::default();

        let inner = tui.tui.add_children_inner(
            tui.params,
            Some(|ui: &mut egui::Ui, container: &TaffyContainerUi| {
                let available_space = ui.available_size();
                let (id, rect) = ui.allocate_space(available_space);
                let response = ui.interact(rect, id, egui::Sense::click());
                let visuals = ui.style().interact(&response);

                let rect = container.full_container_without_border();
                let painter = ui.painter();
                painter.rect_filled(
                    rect.expand(visuals.expansion),
                    visuals.rounding,
                    visuals.weak_bg_fill,
                );
                painter.rect_stroke(rect, visuals.rounding, visuals.bg_stroke);

                *data.borrow_mut() = Some((*visuals, response));
            }),
            |tui, _| {
                let data = data.borrow().as_ref().unwrap().0;
                let egui_style = tui.egui_style_mut();
                egui_style.interaction.selectable_labels = false;
                egui_style.visuals.widgets.inactive = data;
                egui_style.visuals.widgets.noninteractive = data;

                f(tui)
            },
        );

        let data = data.borrow_mut().take().unwrap();
        TuiInnerResponse {
            inner,
            response: data.1,
        }
    }

    /// Add tui node with background that acts as selectable button
    #[must_use = "You should check if the user clicked this with `if ….clicked() { … } "]
    fn selectable<T>(self, selected: bool, f: impl FnOnce(&mut Tui) -> T) -> TuiInnerResponse<T> {
        let tui = self.with_border_style_from_egui_style();
        let data =
            std::cell::RefCell::<Option<(egui::style::WidgetVisuals, egui::Response)>>::default();

        let inner = tui.tui.add_children_inner(
            tui.params,
            Some(|ui: &mut egui::Ui, container: &TaffyContainerUi| {
                let available_space = ui.available_size();
                let (id, rect) = ui.allocate_space(available_space);
                let response = ui.interact(rect, id, egui::Sense::click());
                let visuals = ui.style().interact_selectable(&response, selected);

                let rect = container.full_container_without_border();
                let painter = ui.painter();
                painter.rect_filled(
                    rect.expand(visuals.expansion),
                    visuals.rounding,
                    visuals.weak_bg_fill,
                );
                painter.rect_stroke(rect, visuals.rounding, visuals.bg_stroke);

                *data.borrow_mut() = Some((visuals, response));
            }),
            |tui, _| {
                let data = data.borrow().as_ref().unwrap().0;
                let egui_style = tui.egui_style_mut();
                egui_style.interaction.selectable_labels = false;
                egui_style.visuals.widgets.inactive = data;
                egui_style.visuals.widgets.noninteractive = data;

                f(tui)
            },
        );

        let data = data.borrow_mut().take().unwrap();
        TuiInnerResponse {
            inner,
            response: data.1,
        }
    }

    /// Add tui node as children to this node and draw custom background
    ///
    /// See [`TuiBuilderLogic::add_with_background`] for example
    fn add_with_background_ui<T>(
        self,
        content: impl FnOnce(&mut egui::Ui, &TaffyContainerUi),
        f: impl FnOnce(&mut Tui) -> T,
    ) -> T {
        let tui = self.tui();
        tui.tui
            .add_children_inner(tui.params, Some(content), |tui, _| f(tui))
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

        tui.add_with_background(move |tui| {
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
    fn ui_manual<T>(
        self,
        content: impl FnOnce(&mut Ui, TaffyContainerUi) -> TuiContainerResponse<T>,
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
