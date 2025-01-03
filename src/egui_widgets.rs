use crate::{TuiBuilderLogic, TuiContainerResponse};

use super::{TuiBuilder, TuiWidget};

/// Implement egui widgets for taffy ui
///
/// Idea taken from egui_flex
macro_rules! impl_widget {
    ($($widget:ty),*) => {
        $(
            impl TuiWidget for $widget {
                type Response = egui::Response;

                fn taffy_ui(self, tuib: TuiBuilder) -> Self::Response {
                    tuib.ui_add_manual(|ui| ui.add(self), identity_transform)
                }
            }
        )*
    };
}
impl_widget!(
    egui::Label,
    egui::Checkbox<'_>,
    egui::Image<'_>,
    egui::DragValue<'_>,
    egui::Hyperlink,
    egui::ImageButton<'_>,
    egui::ProgressBar,
    egui::RadioButton,
    egui::Link,
    egui::SelectableLabel,
    egui::Slider<'_>,
    egui::TextEdit<'_>,
    egui::Spinner
);

impl TuiWidget for egui::Button<'_> {
    type Response = egui::Response;

    fn taffy_ui(self, tui: TuiBuilder) -> Self::Response {
        tui.ui_add_manual(
            |ui| ui.centered_and_justified(|ui| ui.add(self)).inner,
            |mut val, _ui| {
                // Button can grow in both dimensions
                val.max_size = val.min_size;
                val.infinite = egui::Vec2b::FALSE;
                val
            },
        )
    }
}

/// Helper function
#[inline]
pub fn identity_transform<T>(
    value: TuiContainerResponse<T>,
    _ui: &egui::Ui,
) -> TuiContainerResponse<T> {
    value
}
