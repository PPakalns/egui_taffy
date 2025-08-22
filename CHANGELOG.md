# Changelog

## 0.8.1

- Fix required space calculations for text. Egui inside intrinsic size doesn't take into account text wrapping (multiple rows).
- Add option to change auto id generation prefix for UI nodes.

## 0.8.0

- Support egui 0.32
- Drop support for `SelectableLabel` (deprecated in egui 0.32)

## 0.7.1

- Avoid allocating space for widgets where simple `egui::Ui::interact` call is sufficient.
- Avoid egui::Style clone on every interactive element (button, selectable, clickable, etc.)

## 0.7.0

- Support egui 0.31
- Added `tui.clickable` to create clickable region
- Inner code refactored to use non generic inner functions where possible (Idea taken from rust std).
- Avoid heap allocations during type erasure using `stackbox` crate.
- Added helper function to access egui_ctx. `tui.egui_ctx()`.
- Expose reusable function to set up egui style for nodes. (98cdbea12697c8868297f945b2d9dc0cf0839dd0)

## 0.6

This release improves the element background drawing API and exposes the internal Taffy state, allowing for the implementation of additional functionalities such as virtual grids and table backgrounds.

- Support for virtual table rows, enabling the rendering of tables with millions of rows while only drawing the visible ones. See demo.
- Added MSRV 1.81. (PR: [#10](https://github.com/PPakalns/egui_taffy/pull/10) by @boxofrox)
- Exposed the internal Taffy tree state for accessing detailed layout information.
- Web demo available. (PR: [#8](https://github.com/PPakalns/egui_taffy/pull/8) by @TheRustyPickle)
- Added the `tui.mut_egui_style(|style| { ... })` function to mutate the Egui style.
- Introduced the `tui.colored_button(color, |tui| { ... })` helper function.
- Improved support for `egui::ProgressBar` sizing.
- Added the `tui.add_with_background_color(...)` helper function.
- Taffy 0.7.3 is now required.
- API changes for drawing background UI.

## 0.5

This release adds support for scrollable elements, handling the overflow style parameter, and sticky elements!

- Added support for elements that can scroll (`overflow: scroll`). Egui_taffy automatically adds `egui::ScrollArea` when `taffy::Overflow:Scroll` is set.
- The `add_with_border`, `button`, and `selectable` methods now set the border size value from `egui::Style` in `taffy::Style` if the border size was set to the default value (`Rect::zero()`).
- Added support for sticky elements.
- Added support for all `taffy::Overflow` values: Visible, Clip, Hidden, Scroll.
- Added examples for all overflow settings: Visible, Clip, Hidden, Scroll.
- Added examples for sticky rows and columns in a scrollable grid.
- The `add_scroll_area` family of functions has been renamed from "add" to "ui" to imply that the inner closure takes `egui::Ui`.
- The `add_with_background` function now takes an additional argument (`&TaffyContainerUi`), which provides more precise layout information for drawing the background.
- Added the `tui.colored_label(color, label)` helper method.

## 0.4

- Support for Egui 0.30.

## 0.3

This release adds support for more granular interaction with the underlying `egui::Ui`. When creating child elements, you can now provide additional settings that are passed to `egui::UiBuilder` (e.g., `egui::Layout`, `egui::Style`, `egui::TextWrapMode`, Disable descendant UI).

- Removed the lifetime requirement for `Tui` (previously `Tui<'a>`).
- Added a shorthand function for adding a label with "strong" coloring: `tui.strong("label");`.
- Added a helper function to set the wrap mode for child layouts: `tui.wrap_mode(egui::TextWrapMode::...).add(|tui| ...)`.
- Added methods to set up child element Egui UI style and layout: `tui.layout(egui::Layout::default()).egui_style(custom_egui_style).add(|tui| ...)`.

## 0.2.1

- Correctly supported the disabling of child elements/nodes (`egui::Ui` disable).

## 0.2

- Updated Taffy to 0.7.
- Added support for classic buttons and selectable buttons.
- Updated the README with information about text wrapping.

## 0.1

Initial functionality.
