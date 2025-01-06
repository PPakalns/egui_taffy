# Changelog

## dev

Release improves element background drawing API and exposes internal taffy state to allow implementing additional functionality.

Added support for virtual table rows. Now it is possible to draw tables with millions of rows in the same size and only visible rows will be drawn.

* Support virtual table rows. See demo.
* Added MSRV 1.81.
* Expose internal taffy tree state.

## 0.5

Release adds support for scrollable elements, handling overflow style parameter and sticky elements!

* Added support for elements that can scroll (`overflow: scroll`).
  egui_taffy automatically adds `egui::ScrollArea` when overflow: `taffy::Overflow:Scroll` is set.
* `add_with_border`, `button`, `selectable` methods now sets border_size value from egui::Style in taffy::Style if border size was set to default value (`Rect::zero()`).
* Added support for sticky elements.
* Added support for all `taffy::Overflow` values: Visible, Clip, Hidden, Scroll.
* Added example for all overflow settings: Visible, Clip, Hidden, Scroll.
* Added example for sticky row, column in scrollable grid.
* `add_scroll_area` family of functions prefix was changed from "add"" to "ui" to imply that inner closure
  takes `egui::Ui`.
* `add_with_background`: background drawing function now takes additional argument (`&TaffyContainerUi`) 
  which contains more precise information about layout that can be used to draw background.
* Added `tui.colored_label(color, label)` helper method.

## 0.4

* Support egui 0.30

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
