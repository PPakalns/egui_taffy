# egui_taffy: Egui Taffy Ui

[![egui_version](https://img.shields.io/badge/egui-0.30-blue)](https://github.com/emilk/egui)
[![taffy_version](https://img.shields.io/badge/taffy-0.7-blue)](https://github.com/DioxusLabs/taffy)
[![Latest version](https://img.shields.io/crates/v/egui_taffy.svg)](https://crates.io/crates/egui_taffy)
[![Documentation](https://docs.rs/egui_taffy/badge.svg)](https://docs.rs/egui_taffy)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/egui_taffy.svg)](https://crates.io/crates/egui_taffy)

Flexible egui layout library that supports CSS Block, Flexbox, Grid layouts. It uses high-performance [taffy](https://github.com/DioxusLabs/taffy) library under the hood.

## Version compatibility

| egui_taffy | egui | taffy |
| ---      | ---  | ---   |
| 0.4      | 0.30 | 0.7   |
| 0.3      | 0.29 | 0.7   | 
| 0.2      | 0.29 | 0.7   | 
| 0.1      | 0.29 | 0.6   | 

To use add `egui_taffy` to your project dependencies in `Cargo.toml` file.

See [CHANGELOG](./CHANGELOG.md) for changes between versions.

## Examples

Check out `./examples/demo.rs` (cargo run --example demo).

### Flex wrap demo example:

```rs
egui::Window::new("Flex wrap demo").show(ctx, |ui| {
    tui(ui, ui.id().with("demo"))
        .reserve_available_space() // Reserve full space of this window for layout
        .style(Style {
            flex_direction: taffy::FlexDirection::Column,
            align_items: Some(taffy::AlignItems::Stretch),
            ..default_style()
        })
        .show(|tui| {
            // Add egui ui as node
            tui.ui(|ui| {
                ui.label("Hello from egui ui!");
                ui.button("Egui button");
            });

            // Add egui widgets directly to UI that implements [`TuiWidget`] trait
            tui.ui_add(egui::Label::new("label"));
            tui.ui_add(egui::Button::new("button"));
            // Or use couple of supported helper function
            tui.separator();
            tui.label("Text");

            // You can add custom style or unique id to every element that is added to the ui
            // by calling id, style, mut_style methods on it first using builder pattern

            // Provide full style
            tui.style(Style {
                align_self: Some(taffy::AlignItems::Center),
                ..Default::default()
            })
            .label("Centered text");

            tui.style(default_style())
                .mut_style(|style| {
                    // Modify one field of the style
                    style.align_self = Some(taffy::AlignItems::End);
                })
                .label("Right aligned text");

            // You can add elements with custom background using add_with_ family of methods
            tui.add_with_border(|tui| {
                tui.label("Text with border");
            });

            tui.separator();

            tui.style(Style {
                flex_wrap: taffy::FlexWrap::Wrap,
                justify_items: Some(taffy::AlignItems::Stretch),
                ..default_style()
            })
            .add(|tui| {
                for word in FLEX_ITEMS {
                    tui.style(default_style()).add_with_border(|tui| {
                        tui.label(word);
                    });
                }
            });
        });
});
```
Preview:

![flex_wrap_demo](https://github.com/user-attachments/assets/0d6ca8cd-dc5b-4f06-aa2e-5a9e5be69bfb)

### Button example

![button_demo](https://github.com/user-attachments/assets/b15875d2-a92e-4dbc-8282-1d9d8fbc1565)

### Grid example

![grid_demo](https://github.com/user-attachments/assets/f72a73f1-c2d3-4d05-869a-84a334cede37)

### Flex example

![flex_demo](https://github.com/user-attachments/assets/7c46e66f-ca01-4dcb-97e6-d8e9a70cd8c7)

#### Grow demo

![grow_demo](https://github.com/user-attachments/assets/967f1de3-7918-46b8-9033-ab9c6928816e)

## Egui options

### Max passes

For best visual look you should enable egui multiple passes support so layout can be immediately recalculated upon some changes.

```rs
ctx.options_mut(|options| {
    options.max_passes = std::num::NonZeroUsize::new(2).unwrap();
});
```

If integrating with egui implementations such as `bevy_egui`, for egui multipass (request_discard) functionality to work you need to use special approach. See `bevy_egui` `simple_multipass` example for such case.

### Text wrapping

By default egui text wrapping tries to utilize as less width as possible. In dynamic layouts it results in text where letters are placed in a column.

Instead you should use one of the following options:
1. Specify minimal width or width for the elements, set text elements to fill width of the parent.
2. Disable text wrapping:
   ```rs
   ctx.style_mut(|style| {
     style.wrap_mode = Some(egui::TextWrapMode::Extend);
   });
   ```


## Inspiration

This crate is inspired by [lucasmerlin](https://github.com/lucasmerlin) previous exploration in this direction by such crates as:
* [egui_taffy](https://github.com/lucasmerlin/hello_egui/)
* [egui_flex](https://github.com/lucasmerlin/hello_egui/)

It combines ideas from both crates and builds upon them to provide easy to use egui like API to write your UI with modern layout support.

It uses egui 0.30 new features intrinsic size and request_discard to request immediate frame redraw (without even drawing the current frame) if layout has changed.

## Contributing

Contributions are welcome. Please add your improvements to examples so that it is easy to see and validate.
