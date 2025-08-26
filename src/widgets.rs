use egui::{Align, Ui, UiBuilder};
use taffy::prelude::{auto, length};

use crate::TuiBuilderParamsAccess;
use crate::{TuiBuilderLogic, TuiWidget};

/// Separator that correctly grows in tui environment in both axis
///
/// Determines draw dimension based on parent node taffy::Style flex direction.
#[derive(Default)]
pub struct TaffySeparator {
    separator: egui::Separator,
}

impl egui::Widget for TaffySeparator {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.add(self.separator)
    }
}

impl TuiWidget for TaffySeparator {
    type Response = egui::Response;

    fn taffy_ui<'a, TuiBuilder: TuiBuilderLogic<'a>>(mut self, tui: TuiBuilder) -> Self::Response {
        let mut tui = tui.tui();
        let flex_direction = tui.builder_tui().current_style().flex_direction;

        let stroke = tui
            .builder_tui()
            .egui_ui()
            .visuals()
            .widgets
            .noninteractive
            .bg_stroke;

        let is_horizontal = match flex_direction {
            taffy::FlexDirection::Row => false,
            taffy::FlexDirection::Column => true,
            taffy::FlexDirection::RowReverse => false,
            taffy::FlexDirection::ColumnReverse => true,
        };

        tui = tui.mut_style(|style| {
            style.align_self = Some(taffy::AlignItems::Stretch);
            let default_separator_space_value = 6.; // Taken from egui::ui
            let space = stroke.width.max(default_separator_space_value);

            let size = match is_horizontal {
                true => taffy::Size {
                    height: length(space),
                    width: auto(),
                },
                false => taffy::Size {
                    height: auto(),
                    width: length(space),
                },
            };
            style.min_size = size;
            style.max_size = size;
            style.size = size;
        });

        let layout = match is_horizontal {
            true => {
                self.separator = self.separator.horizontal();
                egui::Layout::top_down(Align::Center)
            }
            false => {
                self.separator = self.separator.vertical();
                egui::Layout::left_to_right(Align::Center)
            }
        };

        let return_values = tui.add_with_background_ui(
            |ui, container, response| {
                let inner = container.full_container_without_border_and_padding();
                ui.scope_builder(UiBuilder::new().layout(layout).max_rect(inner), |ui| {
                    ui.add(self.separator)
                })
                .inner
            },
            |tui, _| {
                let _ = tui;
            },
        );
        return_values.background
    }
}
