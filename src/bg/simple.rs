use egui::{Response, Visuals, style::WidgetVisuals};

use crate::{AsTuiBuilder, TaffyContainerUi, Tui, TuiBuilder, TuiBuilderLogic, TuiInnerResponse};
use crate::{TuiBuilderParamsAccess, setup_tui_visuals};

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
            background_color: TuiBackgroundValue::Visuals(Box::new(|visuals, _| {
                visuals.panel_fill
            })),
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

impl<'r, T> TuiBuilderLogicWithBackground<'r> for T
where
    T: AsTuiBuilder<'r>,
{
    // Use default implementation
}

/// Logic that provides access to UI node creation with easier to define custom background
pub trait TuiBuilderLogicWithBackground<'r>: TuiBuilderLogic<'r> {
    /// Add tui node as children to this node and draw a custom background and/or a custom border
    /// defined by given [`TuiBackground`]
    ///
    /// Use this function if you want to have full control of [`TuiBackground`] values
    fn bg_add<T>(self, bg: TuiBackground, f: impl FnOnce(&mut Tui) -> T) -> T {
        let tui = self.tui().unpack();
        tui.builder_tui
            .add_child(tui.params, bg.draw(), |tui, _| f(tui))
            .main
    }

    /// Add tui node with a custom background that is clickable
    ///
    /// Use this function if you want to have full control of [`TuiBackground`] values
    fn bg_clickable<T>(
        self,
        bg: TuiBackground,
        f: impl FnOnce(&mut Tui) -> T,
    ) -> TuiInnerResponse<T> {
        let tui = if bg.has_border() {
            self.with_border_style_from_egui_style()
        } else {
            self.tui()
        };

        let TuiBuilder {
            builder_tui,
            params,
        } = tui.unpack();

        let return_values =
            builder_tui.add_child(params, bg.draw_with_response(), |tui, bg_response| {
                setup_tui_visuals(tui, bg_response);
                f(tui)
            });

        TuiInnerResponse {
            inner: return_values.main,
            response: return_values.background,
        }
    }
}
