# Changelog

## 0.3

Release adds support for more granular interaction with underlying `egui::Ui`.
When creating child elements you can provide additional settings that are passed to `egui::UiBuilder`.
(`egui::Layout`, `egui::Style`, `egui::TextWrapMode`, Disable descendant ui).

* Removed lifetime requirement for `Tui` (previously `Tui<'a>`).
* Added shorthand function for adding label with "strong" coloring. `tui.strong("label");`
* Added helper function to set wrap mode for child layout `tui.wrap_mode(egui::TextWrapMode::...).add(|tui| ...)`.
* Added methods to set up child element egui Ui style and layout: `tui.layout(egui::Layout::default()).egui_style(custom_egui_style).add(|tui| ...)`

## 0.2.1

* Correctly support child element/node disabling (egui::Ui disable).

## 0.2

* Taffy updated to 0.7.
* Added support for classic buttons and selectable buttons.
* Added information to README about text wrapping.

## 0.1

Initial functionality
