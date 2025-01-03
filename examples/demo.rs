use egui::Vec2b;
use egui_taffy::{
    tid, tui,
    virtual_tui::{VirtualGridRowHelper, VirtualGridRowHelperParams, VirtualGridRowInfo},
    TuiBuilderLogic,
};
use taffy::{
    prelude::{auto, fr, length, min_content, percent, repeat, span},
    style_helpers, Style,
};

#[derive(Default)]
pub struct State {
    grow_variables: Option<GrowVariables>,
    button_params: ButtonParams,
    show_flex_grid_demo: bool,
    show_flex_demo: bool,
    show_flex_wrap_demo: bool,
    show_grow_demo: bool,
    show_button_demo: bool,
    show_overflow_demo: bool,
    show_grid_sticky_demo: bool,
    show_virtual_grid_demo: bool,
}

fn main() -> eframe::Result {
    let mut state = State::default();

    eframe::run_simple_native("demo", Default::default(), move |ctx, _frame| {
        // Enable multipass rendering upon request without drawing to screen
        //
        // View README for more details
        ctx.options_mut(|options| {
            options.max_passes = std::num::NonZeroUsize::new(3).unwrap();
        });

        // Disable text wrapping
        //
        // egui text layouting tries to utilize minimal width possible
        ctx.style_mut(|style| {
            style.wrap_mode = Some(egui::TextWrapMode::Extend);
        });

        ui_side_panel(ctx, &mut state);

        flex_grid_demo(ctx, &mut state);

        flex_demo(ctx, &mut state);

        flex_wrap_demo(ctx, &mut state);

        grow_demo(ctx, &mut state);

        button_demo(ctx, &mut state);

        overflow_demo(ctx, &mut state);

        grid_sticky(ctx, &mut state);

        virtual_grid_demo(ctx, &mut state);
    })
}

fn ui_side_panel(ctx: &egui::Context, state: &mut State) {
    egui::SidePanel::new(egui::panel::Side::Left, "panel").show(ctx, |ui| {
        tui(ui, ui.id().with("side_panel"))
            .reserve_available_space()
            .style(taffy::Style {
                flex_direction: taffy::FlexDirection::Column,
                align_items: Some(taffy::AlignItems::Stretch),
                gap: length(2.),
                ..Default::default()
            })
            .show(|tui| {
                tui.label("Demos:");
                for (label, show) in [
                    ("Grid demo", &mut state.show_flex_grid_demo),
                    ("Flex demo", &mut state.show_flex_demo),
                    ("Flex wrap demo", &mut state.show_flex_wrap_demo),
                    ("Grow demo", &mut state.show_grow_demo),
                    ("Button demo", &mut state.show_button_demo),
                    ("Overflow demo", &mut state.show_overflow_demo),
                    (
                        "Sticky header and column in grid",
                        &mut state.show_grid_sticky_demo,
                    ),
                    ("Virtual grid row demo", &mut state.show_virtual_grid_demo),
                ] {
                    if tui
                        .style(taffy::Style {
                            padding: length(4.),
                            ..Default::default()
                        })
                        .selectable(*show, |tui| {
                            tui.label(label);
                        })
                        .clicked()
                    {
                        *show = !*show;
                    }
                }
            });
    });
}

fn flex_wrap_demo(ctx: &egui::Context, state: &mut State) {
    let default_style = || Style {
        padding: length(8.),
        gap: length(8.),
        flex_grow: 1.,
        justify_content: Some(taffy::AlignContent::Center),
        ..Default::default()
    };

    egui::Window::new("Flex wrap demo")
        .open(&mut state.show_flex_wrap_demo)
        .show(ctx, |ui| {
            tui(ui, ui.id().with("demo"))
                .reserve_available_space() // Reserve full space of window for this layout
                .style(Style {
                    flex_direction: taffy::FlexDirection::Column,
                    align_items: Some(taffy::AlignItems::Stretch),
                    ..default_style()
                })
                .show(|tui| {
                    // Add egui ui as nodes
                    tui.ui(|ui| {
                        ui.label("Hello from egui ui!");
                        let _ = ui.button("Egui button");
                    });

                    // Add egui widgets directly to UI that implements [`TuiWidget`] trait
                    tui.ui_add(egui::Label::new("label"));
                    tui.ui_add(egui::Button::new("button"));
                    tui.separator();
                    tui.label("Left aligned text");

                    // You can add custom style or unique id to every element that is added to ui
                    // by calling id, style, mut_style methods on it first using builder pattern

                    // Provide full style
                    tui.style(Style {
                        align_self: Some(taffy::AlignItems::Center),
                        ..Default::default()
                    })
                    .egui_layout(egui::Layout::default().with_cross_align(egui::Align::Center))
                    .label("Centered text");

                    tui.style(default_style())
                        .mut_style(|style| {
                            // Modify one field of the style
                            style.align_self = Some(taffy::AlignItems::End);
                        })
                        .egui_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT))
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
}

fn flex_grid_demo(ctx: &egui::Context, state: &mut State) {
    egui::Window::new("Grid demo")
        .open(&mut state.show_flex_grid_demo)
        .show(ctx, |ui| {
            // Style rules can be defined as functions and applied with
            // [`TuiBuilder::mut_style`] function.
            let align_flex_content_in_center = |style: &mut Style| {
                // Align content in center in flexbox layout
                style.justify_content = Some(taffy::JustifyContent::Center);
                style.align_items = Some(taffy::AlignItems::Center);
            };

            // Initialize Tui layout (Taffy ui layout)
            tui(ui, "grid")
                .reserve_available_space()
                .style(Style {
                    display: taffy::Display::Grid,

                    // All columns except last one has the same size
                    grid_template_columns: vec![fr(1.), fr(1.), fr(1.), fr(1.), fr(1.), fr(1.5)],
                    // All rows has the same size
                    grid_template_rows: vec![repeat("auto-fill", vec![fr(1.)])],

                    gap: length(8.),

                    // Fill all available parent space
                    size: percent(1.),

                    // Stretch grid cells by default to fill space
                    align_items: Some(taffy::AlignItems::Stretch),
                    justify_items: Some(taffy::AlignItems::Stretch),

                    ..Default::default()
                })
                .show(|tui| {
                    tui.style(Style {
                        grid_column: span(5),
                        ..Default::default()
                    })
                    .add_with_border(|tui| {
                        tui.ui(|ui| {
                            // Add egui ui as child node to the layout
                            ui.label("Col span 5");
                        });
                    });

                    tui.style(Style {
                        grid_row: span(6),
                        ..Default::default()
                    })
                    .add_with_border(|tui| {
                        tui.ui(|ui| {
                            ui.label("Row span 6");
                        });
                    });

                    let align_list = [
                        taffy::AlignItems::Start,
                        taffy::AlignItems::Center,
                        taffy::AlignItems::End,
                        taffy::AlignItems::Stretch,
                    ];

                    tui.add(|_tui| {
                        //Empty cell
                    });

                    for header in align_list {
                        tui.mut_style(align_flex_content_in_center)
                            .add_with_border(|tui| {
                                tui.label(format!("{:?}", header));
                            });
                    }

                    for align_item in align_list {
                        tui.add_with_border(|tui| {
                            tui.label(format!("{:?}", align_item));
                        });

                        for justify_item in align_list {
                            tui.style(Style {
                                justify_self: Some(justify_item),
                                align_self: Some(align_item),

                                padding: length(4.),
                                ..Default::default()
                            })
                            .mut_style(align_flex_content_in_center)
                            .add_with_border(|tui| {
                                tui.label(format!("{:?} {:?}", align_item, justify_item));
                            });
                        }
                    }
                });
        });
}

pub struct GrowVariables {
    gap: f32,
    margin: f32,
    padding: f32,
}

fn grow_demo(ctx: &egui::Context, state: &mut State) {
    let GrowVariables {
        gap,
        margin,
        padding,
    } = state.grow_variables.get_or_insert(GrowVariables {
        gap: 8.,
        margin: 0.,
        padding: 8.,
    });

    egui::Window::new("Grow demo")
        .open(&mut state.show_grow_demo)
        .show(ctx, |ui| {
            // You can mix egui ui with
            ui.horizontal(|ui| {
                ui.label("Gap");
                ui.add(egui::Slider::new(gap, 0. ..=50.));
            });

            ui.horizontal(|ui| {
                ui.label("Margin");
                ui.add(egui::Slider::new(margin, 0. ..=50.));
            });

            ui.horizontal(|ui| {
                ui.label("Padding");
                ui.add(egui::Slider::new(padding, 0. ..=50.));
            });

            let default_style = || Style {
                padding: length(*padding),
                margin: length(*margin),
                gap: length(*gap),
                ..Default::default()
            };

            // taffy based ui
            tui(ui, ui.id().with("demo"))
                .reserve_available_space()
                .style(Style {
                    flex_direction: taffy::FlexDirection::Column,
                    size: percent(1.),
                    justify_items: Some(taffy::AlignItems::Center),
                    align_items: Some(taffy::AlignItems::Center),
                    ..default_style()
                })
                .show(|tui| {
                    for grow in 0..4 {
                        tui.style(Style {
                            flex_grow: grow as f32,
                            align_items: Some(taffy::AlignItems::Center),
                            ..default_style()
                        })
                        .add_with_border(|tui| {
                            tui.label(format!("Grow {}", grow));
                        });
                    }

                    tui.style(Style {
                        flex_grow: 6.,
                        align_self: Some(taffy::AlignItems::Stretch),

                        align_items: Some(taffy::AlignItems::Stretch),
                        ..default_style()
                    })
                    .add_with_border(|tui| {
                        for grow in 0..4 {
                            tui.style(Style {
                                flex_grow: grow as f32,
                                align_items: Some(taffy::AlignItems::Center),
                                justify_content: Some(taffy::AlignContent::Center),
                                ..default_style()
                            })
                            .add_with_border(|tui| {
                                tui.label(format!("Grow {}", grow));
                            });
                        }
                    });
                });
        });
}

fn flex_demo(ctx: &egui::Context, state: &mut State) {
    egui::Window::new("Flex demo")
        .scroll(Vec2b { x: true, y: true })
        .open(&mut state.show_flex_demo)
        .show(ctx, |ui| {
            let default_style = || Style {
                gap: length(8.),
                padding: length(8.),
                ..Default::default()
            };

            tui(ui, ui.id().with("demo"))
                .reserve_available_width()
                .style(Style {
                    flex_direction: taffy::FlexDirection::Column,
                    min_size: taffy::Size {
                        width: percent(1.),
                        height: auto(),
                    },
                    align_items: Some(taffy::AlignItems::Stretch),
                    max_size: percent(1.),
                    gap: length(8.),
                    ..Default::default()
                })
                .show(|tui| {
                    for (justify_content, flex_grow) in [
                        (taffy::AlignContent::Start, 0.),
                        (taffy::AlignContent::End, 0.),
                        (taffy::AlignContent::Stretch, 0.),
                        (taffy::AlignContent::Stretch, 1.),
                        (taffy::AlignContent::Center, 0.),
                        (taffy::AlignContent::SpaceBetween, 0.),
                        (taffy::AlignContent::SpaceAround, 0.),
                    ] {
                        tui.style(Style {
                            flex_direction: taffy::FlexDirection::Row,
                            min_size: taffy::Size {
                                width: auto(),
                                height: length(100.),
                            },
                            ..default_style()
                        })
                        .add_with_border(|tui| {
                            tui.style(Style {
                                flex_direction: taffy::FlexDirection::Column,
                                size: taffy::Size {
                                    width: length(200.),
                                    height: auto(),
                                },
                                flex_shrink: 0.,
                                ..Default::default()
                            })
                            .add(|tui| {
                                tui.label(format!("Justify items: {:?}", justify_content));
                                tui.label(format!("Flex grow: {:?}", flex_grow));
                                tui.label("Align self:");
                            });

                            tui.style(Style {
                                flex_direction: taffy::FlexDirection::Row,
                                justify_content: Some(justify_content),
                                flex_grow: 1.,
                                min_size: taffy::Size {
                                    width: auto(),
                                    height: length(100.),
                                },
                                ..default_style()
                            })
                            .add_with_border(|tui| {
                                for align in [
                                    taffy::AlignItems::Start,
                                    taffy::AlignItems::End,
                                    taffy::AlignItems::Center,
                                    taffy::AlignItems::Stretch,
                                ] {
                                    tui.style(Style {
                                        align_self: Some(align),
                                        flex_grow,
                                        ..Default::default()
                                    })
                                    .ui_add(egui::Button::new(format!("{:?}", align)));
                                }
                            });
                        });
                    }
                });
        });
}

const FLEX_ITEMS: [&str; 18] = [
    "Lorem",
    "ipsum",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipiscing",
    "elit",
    "Etiam",
    "orci",
    "velit",
    "suscipit",
    "in",
    "tortor",
    "id",
    "ornare",
    "fringilla",
    "tortor",
];

#[derive(Default)]
struct ButtonParams {
    counter: u32,
    selected: bool,
}

fn button_demo(ctx: &egui::Context, state: &mut State) {
    let params = &mut state.button_params;
    egui::Window::new("Button demo")
        .scroll(Vec2b { x: true, y: true })
        .open(&mut state.show_button_demo)
        .show(ctx, |ui| {
            tui(ui, ui.id().with("button demo"))
                .reserve_available_width()
                .style(Style {
                    flex_direction: taffy::FlexDirection::Column,
                    min_size: taffy::Size {
                        width: percent(1.),
                        height: auto(),
                    },
                    align_items: Some(taffy::AlignItems::Stretch),
                    max_size: percent(1.),
                    gap: length(8.),
                    padding: length(8.),
                    ..Default::default()
                })
                .show(|tui| {
                    let align_list = [
                        taffy::AlignItems::Start,
                        taffy::AlignItems::Center,
                        taffy::AlignItems::End,
                        taffy::AlignItems::Stretch,
                    ];

                    let response = tui
                        .style(taffy::Style {
                            flex_direction: taffy::FlexDirection::Column,
                            align_items: Some(taffy::AlignItems::Center),
                            padding: length(8.),
                            ..Default::default()
                        })
                        .button(|tui| {
                            tui.label("Button");

                            for align_item in align_list {
                                tui.style(Style {
                                    flex_direction: taffy::FlexDirection::Column,
                                    align_self: Some(align_item),
                                    padding: length(4.),
                                    ..Default::default()
                                })
                                .add(|tui| {
                                    tui.style(taffy::Style {
                                        align_self: Some(taffy::AlignItems::Center),
                                        ..Default::default()
                                    })
                                    .label(format!("{:?}", align_item));
                                });
                            }
                        });

                    if response.clicked() {
                        params.counter += 1;
                    }

                    tui.label(format!("Button clicked {} times", params.counter));

                    tui.separator();

                    let response = tui
                        .style(taffy::Style {
                            flex_direction: taffy::FlexDirection::Column,
                            align_items: Some(taffy::AlignItems::Center),
                            padding: length(8.),
                            ..Default::default()
                        })
                        .selectable(params.selected, |tui| {
                            tui.label("Selectable button");

                            for align_item in align_list {
                                tui.style(Style {
                                    flex_direction: taffy::FlexDirection::Column,
                                    align_self: Some(align_item),
                                    padding: length(4.),
                                    ..Default::default()
                                })
                                .add(|tui| {
                                    tui.style(taffy::Style {
                                        align_self: Some(taffy::AlignItems::Center),
                                        ..Default::default()
                                    })
                                    .label(format!("{:?}", align_item));
                                });
                            }
                        });
                    if response.clicked() {
                        params.selected = !params.selected;
                    }

                    tui.label(format!("Selected: {}", params.selected));
                });
        });
}

fn overflow_demo(ctx: &egui::Context, state: &mut State) {
    egui::Window::new("Overflow demo")
        .scroll(Vec2b { x: true, y: true })
        .open(&mut state.show_overflow_demo)
        .show(ctx, |ui| {
            tui(ui, ui.id().with("overflow demo"))
                .reserve_available_width()
                .style(Style {
                    flex_direction: taffy::FlexDirection::Row,
                    align_items: Some(taffy::AlignItems::Center),
                    gap: length(16.),
                    ..Default::default()
                })
                .show(|tui| {
                    for overflow in [
                        taffy::Overflow::Visible,
                        taffy::Overflow::Clip,
                        taffy::Overflow::Hidden,
                        taffy::Overflow::Scroll,
                    ] {
                        tui.style(taffy::Style {
                            flex_direction: taffy::FlexDirection::Column,
                            overflow: taffy::Point {
                                x: taffy::Overflow::default(),
                                y: overflow,
                            },
                            max_size: taffy::Size {
                                height: length(200.),
                                width: auto(),
                            },
                            padding: length(12.),
                            ..Default::default()
                        })
                        .add_with_border(|tui| {
                            let label = format!("{:?}", overflow);
                            for _ in 0..50 {
                                tui.label(&label);
                            }
                        });
                    }
                });
        });
}

fn grid_sticky(ctx: &egui::Context, state: &mut State) {
    egui::Window::new("Sticky header and column in grid")
        .scroll(Vec2b::FALSE)
        .open(&mut state.show_grid_sticky_demo)
        .show(ctx, |ui| {
            tui(ui, ui.id().with("sticky grid demo"))
                .reserve_available_space()
                .style(taffy::Style {
                    size: percent(1.),
                    ..Default::default()
                })
                .show(|tui| {
                    let style = tui.egui_style_mut();
                    style.visuals.widgets.noninteractive.rounding = egui::Rounding::ZERO;

                    let cell_style = taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        align_items: Some(taffy::AlignItems::Center),
                        justify_content: Some(taffy::AlignContent::SpaceAround),
                        padding: length(16.),
                        ..Default::default()
                    };

                    let columns = 16i16;
                    let rows = 16i16;

                    tui.style(taffy::Style {
                        overflow: taffy::Point {
                            x: taffy::Overflow::Scroll,
                            y: taffy::Overflow::Scroll,
                        },
                        size: percent(1.),
                        max_size: percent(1.),
                        display: taffy::Display::Grid,
                        align_items: Some(taffy::AlignItems::Stretch),
                        justify_items: Some(taffy::AlignItems::Stretch),
                        grid_template_rows: vec![auto(); (rows + 1) as usize],
                        grid_template_columns: vec![auto(); (columns + 1) as usize],
                        ..Default::default()
                    })
                    .add(|tui| {
                        for i in 1..rows {
                            for j in 1..columns {
                                tui.style(taffy::Style {
                                    grid_column: style_helpers::line(j + 1),
                                    grid_row: style_helpers::line(i + 1),

                                    ..cell_style.clone()
                                })
                                .add_with_border(|tui| {
                                    tui.label(format!("Cell {} {}", i, j));
                                });
                            }
                        }

                        // Header styling
                        let style = tui.egui_style_mut();
                        style.visuals.panel_fill = egui::Color32::DARK_BLUE;

                        for i in 1..columns {
                            tui.sticky([false, true].into())
                                .style(taffy::Style {
                                    grid_column: style_helpers::line(i + 1),
                                    grid_row: style_helpers::line(1),

                                    ..cell_style.clone()
                                })
                                .add_with_background(|tui| {
                                    tui.label(format!("Header {}", i));
                                });
                        }

                        for i in 1..rows {
                            tui.sticky([true, false].into())
                                .style(taffy::Style {
                                    grid_column: style_helpers::line(1),
                                    grid_row: style_helpers::line(i + 1),

                                    ..cell_style.clone()
                                })
                                .add_with_background(|tui| {
                                    tui.label(format!("Row header {}", i));
                                });
                        }

                        tui.sticky(true.into())
                            .style(taffy::Style {
                                grid_column: style_helpers::line(1),
                                grid_row: style_helpers::line(1),

                                ..cell_style.clone()
                            })
                            .add_with_background(|tui| {
                                tui.label("Top left");
                            });
                    });
                });
        });
}

fn virtual_grid_demo(ctx: &egui::Context, state: &mut State) {
    egui::Window::new("Virtual grid row demo")
        .open(&mut state.show_virtual_grid_demo)
        .show(ctx, |ui| {
            tui(ui, ui.id().with("virtual_grid"))
                .reserve_available_space()
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Column,
                    size: percent(1.),
                    max_size: percent(1.),
                    ..Default::default()
                })
                .show(|tui| {
                    tui.style(taffy::Style {
                        display: taffy::Display::Grid,
                        overflow: taffy::Point {
                            x: taffy::Overflow::Visible,
                            y: taffy::Overflow::Scroll,
                        },
                        grid_template_columns: vec![auto(), auto()],
                        size: taffy::Size {
                            width: percent(1.),
                            height: auto(),
                        },
                        max_size: percent(1.),
                        grid_auto_rows: vec![min_content()],
                        ..Default::default()
                    })
                    .add(|tui| {
                        let header_row_count = 2;

                        VirtualGridRowHelper::show(
                            VirtualGridRowHelperParams {
                                header_row_count,
                                row_count: 100000,
                            },
                            tui,
                            |tui, info| {
                                let mut idgen = info.id_gen();
                                let mut_grid_row_param = info.grid_row_setter();
                                let mut container = None;

                                for cidx in 1..=2 {
                                    tui.id(idgen())
                                        .mut_style(&mut_grid_row_param)
                                        .mut_style(|style| {
                                            style.padding = length(2.);
                                        })
                                        .add_ext(|tui, cont| {
                                            let _ = tui
                                                .style(taffy::Style {
                                                    padding: length(4.),
                                                    ..Default::default()
                                                })
                                                .button(|tui| {
                                                    tui.label(format!("Cell {} {}", info.idx, cidx))
                                                });
                                            container = Some(cont);
                                        });
                                }

                                VirtualGridRowInfo {
                                    container: container.unwrap(),
                                }
                            },
                        );

                        tui.sticky([false, true].into())
                            .style(taffy::Style {
                                flex_direction: taffy::FlexDirection::Column,
                                grid_row: style_helpers::line(1),
                                padding: length(4.),
                                align_items: Some(taffy::AlignItems::Center),
                                grid_column: span(2),
                                ..Default::default()
                            })
                            .id(tid(("header", 1)))
                            .add_with_background_color(|tui| {
                                tui.label("Colspan 2 header");
                            });

                        for ridx in 2..=header_row_count {
                            for idx in 0..2 {
                                tui.sticky([false, true].into())
                                    .style(taffy::Style {
                                        grid_row: style_helpers::line(ridx as i16),
                                        padding: length(4.),
                                        ..Default::default()
                                    })
                                    .id(tid(("header", ridx, idx)))
                                    .add_with_background_color(|tui| {
                                        tui.label(format!("Header {} {}", ridx, idx));
                                    });
                            }
                        }
                    });
                });
        });
}
