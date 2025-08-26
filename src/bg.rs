use egui::Response;

use crate::TaffyContainerUi;

/// Types that can draw background
///
/// [`()`] type draws empty background.
pub trait BackgroundDraw {
    /// Value returned by background drawing functionality
    type ReturnValue;

    /// Implements background drawing functionality
    fn draw(
        self,
        ui: &mut egui::Ui,
        container: &TaffyContainerUi,
        response: Option<&egui::Response>,
    ) -> Self::ReturnValue;
}

impl<T, B> BackgroundDraw for T
where
    T: FnOnce(&mut egui::Ui, &TaffyContainerUi, Option<&egui::Response>) -> B,
{
    type ReturnValue = B;

    #[inline]
    fn draw(
        self,
        ui: &mut egui::Ui,
        container: &TaffyContainerUi,
        response: Option<&egui::Response>,
    ) -> Self::ReturnValue {
        self(ui, container, response)
    }
}

impl BackgroundDraw for TuiBackground<'_> {
    type ReturnValue = ();

    fn draw(
        self,
        ui: &mut egui::Ui,
        container: &TaffyContainerUi,
        response: Option<&egui::Response>,
    ) -> Self::ReturnValue {
        self.draw_internal(ui, container, response)
    }
}

impl BackgroundDraw for () {
    type ReturnValue = ();

    #[inline]
    fn draw(
        self,
        ui: &mut egui::Ui,
        container: &TaffyContainerUi,
        _response: Option<&egui::Response>,
    ) -> Self::ReturnValue {
        let _ = container;
        let _ = ui;
        ()
    }
}

/// Helper to draw background fills and borders
pub struct TuiBackground<'a> {
    default: TuiBackgroundValue<'a, TuiBackgroundParams<'a>>,
    custom: TuiBackgroundParams<'a>,
}

impl<'a> Default for TuiBackground<'a> {
    fn default() -> Self {
        Self {
            default: Default::default(),
            custom: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct TuiBackgroundParams<'a> {
    background_color: TuiBackgroundValue<'a, egui::Color32>,
    corner_radius: TuiBackgroundValue<'a, egui::CornerRadius>,
    border: Option<TuiBackgroundBorder<'a>>,
}

fn default_background_params<'a, 'b>(
    visuals: &'b egui::style::Visuals,
    widget_visuals: &'b egui::style::WidgetVisuals,
) -> TuiBackgroundParams<'a> {
    TuiBackgroundParams {
        background_color: TuiBackgroundValue::Custom(visuals.panel_fill),
        corner_radius: TuiBackgroundValue::Custom(widget_visuals.corner_radius),
        border: None,
    }
}

impl<'a> TuiBackground<'a> {
    /// Creates a new instance drawing a default background only
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_default(
        mut self,
        default: TuiBackgroundValue<'a, TuiBackgroundParams<'a>>,
    ) -> Self {
        self.default = default;
        self
    }

    /// Draws background by given [`egui::Color32`]
    pub fn with_background_color(mut self, background_color: egui::Color32) -> Self {
        self.custom.background_color = TuiBackgroundValue::Custom(background_color);
        self
    }

    /// Draws background color defined by [`egui::style::Visuals`]
    pub fn with_background_color_by_visuals(
        mut self,
        f: VisualsGetterFn<'a, egui::Color32>,
    ) -> Self {
        self.custom.background_color = TuiBackgroundValue::Visuals(f);
        self
    }

    /// Draws background color depending on [`VisualsResponseGetterFn`]
    pub fn with_background_color_by_response(
        mut self,
        f: VisualsResponseGetterFn<'a, egui::Color32>,
    ) -> Self {
        self.custom.background_color = TuiBackgroundValue::VisualsResponse(f);
        self
    }

    /// Draws default border
    pub fn with_border(mut self) -> Self {
        self.custom.border = Some(TuiBackgroundBorder::default());
        self
    }

    /// Draws a border by given [`egui::Color32`]
    pub fn with_border_color(mut self, color: egui::Color32) -> Self {
        self.custom.border.get_or_insert_default().color = TuiBackgroundValue::Custom(color);
        self
    }

    /// Draws a border color defined by [`VisualsGetterFn`]
    pub fn with_border_color_by_visuals(mut self, f: VisualsGetterFn<'a, egui::Color32>) -> Self {
        self.custom.border.get_or_insert_default().color = TuiBackgroundValue::Visuals(f);
        self
    }

    /// Draws a border color defined by [`VisualsResponseGetterFn`]
    pub fn with_border_color_by_response(
        mut self,
        f: VisualsResponseGetterFn<'a, egui::Color32>,
    ) -> Self {
        self.custom.border.get_or_insert_default().color = TuiBackgroundValue::VisualsResponse(f);
        self
    }

    /// Draws border with given width
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.custom.border.get_or_insert_default().width = TuiBackgroundValue::Custom(width);
        self
    }

    /// Draws a border width defined by [`VisualsGetterFn`]
    pub fn with_border_width_by_visuals(mut self, f: VisualsGetterFn<'a, f32>) -> Self {
        self.custom.border.get_or_insert_default().width = TuiBackgroundValue::Visuals(f);
        self
    }

    /// Draws a border width defined by [`VisualsResponseGetterFn`]
    pub fn with_border_width_by_response(mut self, f: VisualsResponseGetterFn<'a, f32>) -> Self {
        self.custom.border.get_or_insert_default().width = TuiBackgroundValue::VisualsResponse(f);
        self
    }

    /// Draws corner radius with given radius
    pub fn with_corner_radius(mut self, radius: impl Into<egui::CornerRadius>) -> Self {
        self.custom.corner_radius = TuiBackgroundValue::Custom(radius.into());
        self
    }

    /// Draws background with given [`egui::CornerRadius`] defined by [`VisualsGetterFn`]
    pub fn with_corner_radius_by_visuals(
        mut self,
        f: VisualsGetterFn<'a, egui::CornerRadius>,
    ) -> Self {
        self.custom.corner_radius = TuiBackgroundValue::Visuals(f);
        self
    }

    /// Draws background with given [`egui::CornerRadius`] defined by [`VisualsResponseGetterFn`]
    pub fn with_corner_radius_by_response(
        mut self,
        f: VisualsResponseGetterFn<'a, egui::CornerRadius>,
    ) -> Self {
        self.custom.corner_radius = TuiBackgroundValue::VisualsResponse(f);
        self
    }

    pub(super) fn has_border(&self) -> bool {
        self.custom.border.is_some()
    }

    // Internal function to draw content of background.
    fn draw_internal(
        self,
        ui: &egui::Ui,
        container: &TaffyContainerUi,
        response: Option<&Response>,
    ) {
        let style = ui.style();
        let visuals = &style.visuals;

        let widget_visuals = if let Some(response) = response {
            ui.style().interact(&response)
        } else {
            ui.style().visuals.noninteractive()
        };

        let rect = container.full_container();

        // Helper to get value out from `TuiBackgroundValue`
        fn match_value<T>(
            visuals: &egui::style::Visuals,
            widget_visuals: &egui::style::WidgetVisuals,
            response: Option<&Response>,
            value: TuiBackgroundValue<T>,
        ) -> Option<T> {
            match value {
                TuiBackgroundValue::Default => None,
                TuiBackgroundValue::Custom(value) => Some(value),
                TuiBackgroundValue::Visuals(f) => Some((*f.func())(visuals, widget_visuals)),
                TuiBackgroundValue::VisualsResponse(f) => match response {
                    Some(r) => Some((*f.func())(visuals, widget_visuals, r)),
                    None => unreachable!("never called without a response"),
                },
            }
        }

        fn match_value_or_fallback<T: Copy>(
            visuals: &egui::style::Visuals,
            widget_visuals: &egui::style::WidgetVisuals,
            response: Option<&Response>,
            value: TuiBackgroundValue<T>,
            fallback: Option<TuiBackgroundValue<T>>,
        ) -> Option<T> {
            match match_value(visuals, widget_visuals, response, value) {
                Some(value) => Some(value),
                None => match fallback {
                    Some(fallback) => match_value(visuals, widget_visuals, response, fallback),
                    None => None,
                },
            }
        }

        let base = match_value(visuals, widget_visuals, response, self.default);

        let (fallback_background_color, fallback_corner_radius, fallback_border) = match base {
            Some(TuiBackgroundParams {
                background_color,
                corner_radius,
                border,
            }) => (Some(background_color), Some(corner_radius), border),
            None => (None, None, None),
        };

        // optional fill
        let fill = match_value_or_fallback(
            visuals,
            widget_visuals,
            response,
            self.custom.background_color,
            fallback_background_color,
        );

        // optional stroke
        let stroke = self
            .custom
            .border
            .or_else(|| {
                Some(TuiBackgroundBorder {
                    color: TuiBackgroundValue::Default,
                    width: TuiBackgroundValue::Default,
                })
            })
            .and_then(|border| {
                let (fallback_color, fallback_width) = match fallback_border {
                    Some(TuiBackgroundBorder { color, width }) => (Some(color), Some(width)),
                    None => (None, None),
                };

                let color = match_value_or_fallback(
                    visuals,
                    widget_visuals,
                    response,
                    border.color,
                    fallback_color,
                )?;
                let width = match_value_or_fallback(
                    visuals,
                    widget_visuals,
                    response,
                    border.width,
                    fallback_width,
                )?;
                Some(egui::Stroke { color, width })
            });

        let corner_radius = match_value_or_fallback(
            visuals,
            widget_visuals,
            response,
            self.custom.corner_radius,
            fallback_corner_radius,
        )
        .unwrap_or_default();

        match (fill, stroke) {
            (None, None) => {}
            (None, Some(stroke)) => {
                ui.painter()
                    .rect_stroke(rect, corner_radius, stroke, egui::StrokeKind::Inside);
            }
            (Some(fill), None) => {
                ui.painter().rect_filled(rect, corner_radius, fill);
            }
            (Some(fill), Some(stroke)) => {
                ui.painter()
                    .rect(rect, corner_radius, fill, stroke, egui::StrokeKind::Inside);
            }
        }
    }
}

struct TuiBackgroundBorder<'a> {
    color: TuiBackgroundValue<'a, egui::Color32>,
    width: TuiBackgroundValue<'a, f32>,
}

impl Default for TuiBackgroundBorder<'_> {
    fn default() -> Self {
        Self {
            color: TuiBackgroundValue::Visuals(BoxRef::Ref(&|_, widget_visuals| {
                widget_visuals.bg_stroke.color
            })),
            width: TuiBackgroundValue::Visuals(BoxRef::Ref(&|_, widget_visuals| {
                widget_visuals.bg_stroke.width
            })),
        }
    }
}

/// Generic structure of values to draw a background by [`TuiBackground`]
#[derive(Default)]
pub enum TuiBackgroundValue<'a, T: Sized> {
    /// Default
    #[default]
    Default,
    /// Custom value
    Custom(T),
    /// Value defined by [`egui::style::Visuals`] accessible by a getter function
    Visuals(VisualsGetterFn<'a, T>),
    /// Value depending on [`egui::Visuals`] accessible by a getter function
    VisualsResponse(VisualsResponseGetterFn<'a, T>),
}

/// Getter function to get values defined by [`egui::style::Visuals`] and [`egui::style::WidgetVisuals`]
pub type VisualsGetterFn<'a, T> =
    BoxRef<'a, dyn Fn(&egui::Visuals, &egui::style::WidgetVisuals) -> T>;

/// Getter function to get values defined by [`egui::style::Visuals`] and [`egui::style::WidgetVisuals`]
/// It includes a reference to [`egui::Response`] to handle widget states
pub type VisualsResponseGetterFn<'a, T> =
    BoxRef<'a, dyn Fn(&egui::Visuals, &egui::style::WidgetVisuals, &egui::Response) -> T>;

pub enum BoxRef<'a, T: ?Sized> {
    Ref(&'a T),
    Box(Box<T>),
}

impl<'a, T: ?Sized> BoxRef<'a, T> {
    pub fn func(&self) -> &T {
        match self {
            BoxRef::Ref(a) => a,
            BoxRef::Box(a) => &a,
        }
    }
}
