use eframe::egui::{self, Vec2b};
use eframe::{App, Frame};
use egui_taffy::bg::simple::{TuiBackground, TuiBuilderLogicWithBackground};
use egui_taffy::{
    TuiBuilderLogic, taffy, tid, tui,
    virtual_tui::{VirtualGridRowHelper, VirtualGridRowHelperParams},
};
use taffy::{
    prelude::{auto, fr, length, min_content, percent, repeat, span},
    style_helpers,
};

#[derive(Default)]
pub struct MyApp {
    state: State,
}

#[derive(Default)]
pub struct State {
    show_flex_grid_demo: bool,
    show_flex_demo: bool,
    show_flex_wrap_demo: bool,
    show_grow_demo: bool,
    show_button_demo: bool,
    show_overflow_demo: bool,
    show_grid_sticky_demo: bool,
    show_virtual_grid_demo: bool,
    show_background_demo: bool,
    show_holy_grail_demo: bool,

    grow_variables: Option<GrowVariables>,
    button_params: ButtonParams,
    overflow_demo_params: OverflowParams,
}

impl App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let state = &mut self.state;

        // Enable multipass rendering upon request without drawing to screen
        //
        // View README for more details
        ui.options_mut(|options| {
            options.max_passes = std::num::NonZeroUsize::new(3).unwrap();
        });

        // Disable text wrapping
        //
        // egui text layouting tries to utilize minimal width possible
        ui.global_style_mut(|style| {
            style.wrap_mode = Some(egui::TextWrapMode::Extend);
        });

        ui_side_panel(ui, state);

        flex_grid_demo(ui, state);

        flex_demo(ui, state);

        flex_wrap_demo(ui, state);

        grow_demo(ui, state);

        button_demo(ui, state);

        overflow_demo(ui, state);

        grid_sticky(ui, state);

        virtual_grid_demo(ui, state);

        custom_background_demo(ui, state);

        holy_grail_demo(ui, state);
    }
}

fn ui_side_panel(ui: &mut egui::Ui, state: &mut State) {
    egui::panel::Panel::left("panel").show(ui, |ui| {
        tui(ui, ui.scope_id().with("side_panel"))
            .reserve_available_space()
            .style(taffy::Style {
                flex_direction: taffy::FlexDirection::Column,
                align_items: taffy::AlignItems::STRETCH,
                padding: length(4f32),
                gap: length(2f32),
                ..Default::default()
            })
            .show(|tui| {
                tui.heading("Egui Taffy");
                tui.ui(|ui| {
                    ui.hyperlink("https://github.com/PPakalns/egui_taffy");
                });

                tui.separator();

                tui.heading("Demos:");

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
                    ("Background demo", &mut state.show_background_demo),
                    (
                        "Background holy grail demo",
                        &mut state.show_holy_grail_demo,
                    ),
                ] {
                    if tui
                        .style(taffy::Style {
                            padding: length(4f32),
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

fn flex_wrap_demo(ui: &mut egui::Ui, state: &mut State) {
    let default_style = || taffy::Style {
        padding: length(8f32),
        gap: length(8f32),
        flex_grow: 1.,
        justify_content: taffy::AlignContent::CENTER,
        ..Default::default()
    };

    egui::Window::new("Flex wrap demo")
        .open(&mut state.show_flex_wrap_demo)
        .show(ui.ctx(), |ui| {
            tui(ui, ui.scope_id().with("demo"))
                .reserve_available_space() // Reserve full space of window for this layout
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Column,
                    align_items: taffy::AlignItems::STRETCH,
                    size: taffy::Size {
                        width: percent(1f32),
                        height: auto(),
                    },

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
                    tui.style(taffy::Style {
                        align_self: Some(taffy::AlignItems::CENTER),
                        ..Default::default()
                    })
                    .egui_layout(egui::Layout::default().with_cross_align(egui::Align::Center))
                    .label("Centered text");

                    tui.style(default_style())
                        .mut_style(|style| {
                            // Modify one field of the style
                            style.align_self = Some(taffy::AlignItems::END);
                        })
                        .egui_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT))
                        .label("Right aligned text");

                    // You can add elements with custom background using add_with_ family of methods
                    tui.add_with_border(|tui| {
                        tui.label("Text with border");
                    });

                    tui.separator();

                    tui.style(taffy::Style {
                        flex_wrap: taffy::FlexWrap::Wrap,
                        justify_items: taffy::AlignItems::STRETCH,
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

fn flex_grid_demo(ui: &mut egui::Ui, state: &mut State) {
    egui::Window::new("Grid demo")
        .open(&mut state.show_flex_grid_demo)
        .show(ui.ctx(), |ui| {
            // Style rules can be defined as functions and applied with
            // [`TuiBuilder::mut_style`] function.
            let align_flex_content_in_center = |style: &mut taffy::Style| {
                // Align content in center in flexbox layout
                style.justify_content = taffy::JustifyContent::CENTER;
                style.align_items = taffy::AlignItems::CENTER;
            };

            // Initialize Tui layout (Taffy ui layout)
            tui(ui, ui.scope_id().with("grid"))
                .reserve_available_space()
                .style(taffy::Style {
                    display: taffy::Display::Grid,

                    // All columns except last one has the same size
                    grid_template_columns: vec![
                        fr(1f32),
                        fr(1f32),
                        fr(1f32),
                        fr(1f32),
                        fr(1f32),
                        fr(1.5f32),
                    ],
                    // All rows has the same size
                    grid_template_rows: vec![repeat("auto-fill", vec![fr(1f32)])],

                    gap: length(8f32),

                    // Fill all available parent space
                    size: percent(1f32),

                    // Stretch grid cells by default to fill space
                    align_items: taffy::AlignItems::STRETCH,
                    justify_items: taffy::AlignItems::STRETCH,

                    ..Default::default()
                })
                .show(|tui| {
                    tui.style(taffy::Style {
                        grid_column: span(5),
                        ..Default::default()
                    })
                    .add_with_border(|tui| {
                        tui.ui(|ui| {
                            // Add egui ui as child node to the layout
                            ui.label("Col span 5");
                        });
                    });

                    tui.style(taffy::Style {
                        grid_row: span(6),
                        ..Default::default()
                    })
                    .add_with_border(|tui| {
                        tui.ui(|ui| {
                            ui.label("Row span 6");
                        });
                    });

                    let align_list = [
                        taffy::AlignItems::START,
                        taffy::AlignItems::CENTER,
                        taffy::AlignItems::END,
                        taffy::AlignItems::STRETCH,
                    ];

                    tui.add(|_tui| {
                        //Empty cell
                    });

                    for header in align_list {
                        tui.mut_style(align_flex_content_in_center)
                            .add_with_border(|tui| {
                                tui.label(align_items_name(header));
                            });
                    }

                    for align_item in align_list {
                        tui.add_with_border(|tui| {
                            tui.label(align_items_name(align_item));
                        });

                        for justify_item in align_list {
                            tui.style(taffy::Style {
                                justify_self: Some(justify_item),
                                align_self: Some(align_item),

                                padding: length(4f32),
                                ..Default::default()
                            })
                            .mut_style(align_flex_content_in_center)
                            .add_with_border(|tui| {
                                tui.label(format!(
                                    "{} {}",
                                    align_items_name(align_item),
                                    align_items_name(justify_item)
                                ));
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

fn grow_demo(ui: &mut egui::Ui, state: &mut State) {
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
        .show(ui.ctx(), |ui| {
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

            let default_style = || taffy::Style {
                padding: length(*padding),
                margin: length(*margin),
                gap: length(*gap),
                ..Default::default()
            };

            // taffy based ui
            tui(ui, ui.scope_id().with("demo"))
                .reserve_available_space()
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Column,
                    size: percent(1f32),
                    justify_items: taffy::AlignItems::CENTER,
                    align_items: taffy::AlignItems::END,
                    ..default_style()
                })
                .show(|tui| {
                    for grow in 0..4 {
                        tui.style(taffy::Style {
                            flex_grow: grow as f32,
                            align_items: taffy::AlignItems::CENTER,
                            ..default_style()
                        })
                        .add_with_border(|tui| {
                            tui.label(format!("Grow {}", grow));
                        });
                    }

                    tui.style(taffy::Style {
                        flex_grow: 6.,
                        align_self: Some(taffy::AlignItems::STRETCH),

                        align_items: taffy::AlignItems::STRETCH,
                        ..default_style()
                    })
                    .add_with_border(|tui| {
                        for grow in 0..4 {
                            tui.style(taffy::Style {
                                flex_grow: grow as f32,
                                align_items: taffy::AlignItems::CENTER,
                                justify_content: taffy::AlignContent::CENTER,
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

fn flex_demo(ui: &mut egui::Ui, state: &mut State) {
    egui::Window::new("Flex demo")
        .scroll(Vec2b { x: true, y: true })
        .open(&mut state.show_flex_demo)
        .default_width(500.)
        .show(ui.ctx(), |ui| {
            let default_style = || taffy::Style {
                gap: length(8f32),
                padding: length(8f32),
                ..Default::default()
            };

            tui(ui, ui.scope_id().with("demo"))
                .reserve_available_width()
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Column,
                    min_size: taffy::Size {
                        width: percent(1f32),
                        height: auto(),
                    },
                    align_items: taffy::AlignItems::STRETCH,
                    max_size: percent(1f32),
                    gap: length(8f32),
                    ..Default::default()
                })
                .show(|tui| {
                    for (justify_content, flex_grow) in [
                        (taffy::AlignContent::START, 0.),
                        (taffy::AlignContent::END, 0.),
                        (taffy::AlignContent::STRETCH, 0.),
                        (taffy::AlignContent::STRETCH, 1.),
                        (taffy::AlignContent::CENTER, 0.),
                        (taffy::AlignContent::SPACE_BETWEEN, 0.),
                        (taffy::AlignContent::SPACE_AROUND, 0.),
                    ] {
                        tui.style(taffy::Style {
                            flex_direction: taffy::FlexDirection::Row,
                            min_size: taffy::Size {
                                width: auto(),
                                height: length(100f32),
                            },
                            ..default_style()
                        })
                        .add_with_border(|tui| {
                            tui.style(taffy::Style {
                                flex_direction: taffy::FlexDirection::Column,
                                size: taffy::Size {
                                    width: length(200f32),
                                    height: auto(),
                                },
                                flex_shrink: 0.,
                                ..Default::default()
                            })
                            .add(|tui| {
                                tui.label(format!(
                                    "Justify items: {}",
                                    align_content_name(justify_content)
                                ));
                                tui.label(format!("Flex grow: {:?}", flex_grow));
                                tui.label("Align self:");
                            });

                            tui.style(taffy::Style {
                                flex_direction: taffy::FlexDirection::Row,
                                justify_content: justify_content,
                                flex_grow: 1.,
                                min_size: taffy::Size {
                                    width: auto(),
                                    height: length(100f32),
                                },
                                ..default_style()
                            })
                            .add_with_border(|tui| {
                                for align in [
                                    taffy::AlignItems::START,
                                    taffy::AlignItems::END,
                                    taffy::AlignItems::CENTER,
                                    taffy::AlignItems::STRETCH,
                                ] {
                                    tui.style(taffy::Style {
                                        align_self: Some(align),
                                        flex_grow,
                                        ..Default::default()
                                    })
                                    .ui_add(egui::Button::new(align_items_name(align)));
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

fn button_demo(ui: &mut egui::Ui, state: &mut State) {
    let params = &mut state.button_params;
    egui::Window::new("Button demo")
        .scroll(Vec2b { x: true, y: true })
        .open(&mut state.show_button_demo)
        .show(ui.ctx(), |ui| {
            tui(ui, ui.scope_id().with("button demo"))
                .reserve_available_width()
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Column,
                    min_size: taffy::Size {
                        width: percent(1f32),
                        height: auto(),
                    },
                    align_items: taffy::AlignItems::STRETCH,
                    max_size: percent(1f32),
                    gap: length(8f32),
                    padding: length(8f32),
                    ..Default::default()
                })
                .show(|tui| {
                    let align_list = [
                        taffy::AlignItems::START,
                        taffy::AlignItems::CENTER,
                        taffy::AlignItems::END,
                        taffy::AlignItems::STRETCH,
                    ];

                    let response = tui
                        .style(taffy::Style {
                            flex_direction: taffy::FlexDirection::Column,
                            align_items: taffy::AlignItems::CENTER,
                            padding: length(8f32),
                            ..Default::default()
                        })
                        .button(|tui| {
                            tui.label("Button");

                            for align_item in align_list {
                                tui.style(taffy::Style {
                                    flex_direction: taffy::FlexDirection::Column,
                                    align_self: Some(align_item),
                                    padding: length(4f32),
                                    ..Default::default()
                                })
                                .add(|tui| {
                                    tui.style(taffy::Style {
                                        align_self: Some(taffy::AlignItems::CENTER),
                                        ..Default::default()
                                    })
                                    .label(align_items_name(align_item));
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
                            align_items: taffy::AlignItems::CENTER,
                            padding: length(8f32),
                            ..Default::default()
                        })
                        .selectable(params.selected, |tui| {
                            tui.label("Selectable button");

                            for align_item in align_list {
                                tui.style(taffy::Style {
                                    flex_direction: taffy::FlexDirection::Column,
                                    align_self: Some(align_item),
                                    padding: length(4f32),
                                    ..Default::default()
                                })
                                .add(|tui| {
                                    tui.style(taffy::Style {
                                        align_self: Some(taffy::AlignItems::CENTER),
                                        ..Default::default()
                                    })
                                    .label(align_items_name(align_item));
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

fn overflow_demo(ui: &mut egui::Ui, state: &mut State) {
    egui::Window::new("Overflow demo")
        .scroll(Vec2b { x: false, y: true })
        .open(&mut state.show_overflow_demo)
        .show(ui.ctx(), |ui| {
            tui(ui, ui.scope_id().with("overflow demo"))
                .reserve_available_width()
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Column,
                    gap: length(16f32),
                    ..Default::default()
                })
                .show(|tui| {
                    tui.ui(|ui| {
                        ui.heading("UI nodes with non-zero borders and paddings with overflow-y");
                        ui.horizontal(|ui| {
                            ui.label("Count");
                            ui.add(egui::Slider::new(
                                &mut state.overflow_demo_params.count,
                                0..=50,
                            ));
                        });
                    });
                    tui.style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Row,
                        align_items: taffy::AlignItems::CENTER,
                        gap: length(16f32),
                        ..Default::default()
                    })
                    .add(|tui| {
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
                                size: taffy::Size {
                                    height: length(200f32),
                                    width: auto(),
                                },
                                padding: length(6f32),
                                border: taffy::Rect {
                                    left: length(20f32),
                                    right: length(40f32),
                                    top: length(20f32),
                                    bottom: length(40f32),
                                },
                                ..Default::default()
                            })
                            .add_with_taffy_border(|tui| {
                                let label = format!("{:?}", overflow);
                                for _ in 0..state.overflow_demo_params.count {
                                    tui.label(&label);
                                }
                            });
                        }
                    });
                });
        });
}

struct OverflowParams {
    count: usize,
}

impl Default for OverflowParams {
    fn default() -> Self {
        Self { count: 20 }
    }
}

fn grid_sticky(ui: &mut egui::Ui, state: &mut State) {
    egui::Window::new("Sticky header and column in grid")
        .scroll(Vec2b::FALSE)
        .open(&mut state.show_grid_sticky_demo)
        .show(ui.ctx(), |ui| {
            tui(ui, ui.scope_id().with("sticky grid demo"))
                .reserve_available_space()
                .style(taffy::Style {
                    size: percent(1f32),
                    ..Default::default()
                })
                .show(|tui| {
                    let style = tui.egui_style_mut();
                    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;

                    let cell_style = taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        align_items: taffy::AlignItems::CENTER,
                        justify_content: taffy::AlignContent::SPACE_AROUND,
                        padding: length(16f32),
                        ..Default::default()
                    };

                    let columns = 16i16;
                    let rows = 16i16;

                    tui.style(taffy::Style {
                        overflow: taffy::Point {
                            x: taffy::Overflow::Scroll,
                            y: taffy::Overflow::Scroll,
                        },
                        size: percent(1f32),
                        max_size: percent(1f32),
                        display: taffy::Display::Grid,
                        align_items: taffy::AlignItems::STRETCH,
                        justify_items: taffy::AlignItems::STRETCH,
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

fn virtual_grid_demo(ui: &mut egui::Ui, state: &mut State) {
    egui::Window::new("Virtual grid row demo")
        .open(&mut state.show_virtual_grid_demo)
        .show(ui.ctx(), |ui| {
            tui(ui, ui.scope_id().with("virtual_grid"))
                .reserve_available_space()
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Column,
                    size: percent(1f32),
                    max_size: percent(1f32),
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
                            width: percent(1f32),
                            height: auto(),
                        },
                        max_size: percent(1f32),
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

                                if (info.grid_row & 1) != 0 {
                                    for cidx in 1..=2 {
                                        let _ = tui
                                            .id(idgen())
                                            .mut_style(&mut_grid_row_param)
                                            .mut_style(|style| {
                                                style.padding = length(2f32);
                                            })
                                            .button(|tui| {
                                                tui.label(format!("Cell {} {}", info.idx, cidx))
                                            });
                                    }
                                } else {
                                    let _ = tui
                                        .id(idgen())
                                        .mut_style(&mut_grid_row_param)
                                        .mut_style(|style| {
                                            style.padding = length(2f32);
                                            style.grid_column = span(2);
                                            style.justify_content =
                                                taffy::AlignContent::SPACE_AROUND;
                                        })
                                        .button(|tui| {
                                            tui.label(format!("Cell {} - Colspan 2", info.idx))
                                        });
                                }
                            },
                        );

                        tui.sticky([false, true].into())
                            .style(taffy::Style {
                                flex_direction: taffy::FlexDirection::Column,
                                grid_row: style_helpers::line(1),
                                padding: length(4f32),
                                align_items: taffy::AlignItems::CENTER,
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
                                        padding: length(4f32),
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

/// See [`egui_taffy::bg::simple::TuiBackground`] and
/// [`egui_taffy::bg::simple::TuiBuilderLogicWithBackground`] implementations
/// to see how to extend functionality of [`egui_taffy`].`
fn custom_background_demo(ui: &mut egui::Ui, state: &mut State) {
    let params = &mut state.button_params;
    egui::Window::new("Custom background demo")
        .open(&mut state.show_background_demo)
        .default_width(500.)
        .show(ui.ctx(), |ui| {
            tui(ui, ui.scope_id().with("demo"))
                .reserve_available_space()
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Column,
                    justify_content: taffy::JustifyContent::CENTER,
                    align_items: taffy::AlignItems::CENTER,
                    max_size: percent(1f32),
                    size: percent(1f32),
                    gap: length(10f32),
                    ..Default::default()
                })
                .show(|tui| {
                    let box_style = taffy::Style {
                        size: taffy::Size {
                            height: length(200f32),
                            width: length(200f32),
                        },
                        justify_content: taffy::JustifyContent::CENTER,
                        align_items: taffy::AlignItems::CENTER,
                        padding: length(20f32),
                        ..Default::default()
                    };

                    let row_style = taffy::Style {
                        flex_direction: taffy::FlexDirection::Row,
                        gap: length(10f32),
                        padding: length(10f32),
                        ..Default::default()
                    };

                    tui.add(|tui| tui.heading("backgrounds".to_uppercase()));

                    tui.style(row_style.clone()).bg_add(
                        TuiBackground::new().with_background_color(egui::Color32::BLACK),
                        |tui| {
                            tui.style(box_style.clone())
                                .bg_add(TuiBackground::new(), |tui| {
                                    tui.colored_label(
                                        egui::Color32::WHITE,
                                        "bg default".to_uppercase(),
                                    );
                                });

                            tui.style(box_style.clone()).bg_add(
                                TuiBackground::new().with_border(),
                                |tui| {
                                    tui.colored_label(
                                        egui::Color32::WHITE,
                                        "bg + border default".to_uppercase(),
                                    );
                                },
                            );

                            tui.style(box_style.clone()).bg_add(
                                TuiBackground::new()
                                    .with_background_color(egui::Color32::TRANSPARENT),
                                |tui| {
                                    tui.colored_label(
                                        egui::Color32::WHITE,
                                        "bg transparent".to_uppercase(),
                                    );
                                },
                            );

                            tui.style(box_style.clone()).bg_add(
                                TuiBackground::new()
                                    .with_background_color(egui::Color32::DARK_GRAY),
                                |tui| {
                                    tui.colored_label(
                                        egui::Color32::WHITE,
                                        "bg custom".to_uppercase(),
                                    );
                                },
                            );

                            tui.style(box_style.clone()).bg_add(
                                TuiBackground::new()
                                    .with_background_color(egui::Color32::LIGHT_GRAY)
                                    .with_border_color(egui::Color32::WHITE)
                                    .with_border_width(20.)
                                    .with_corner_radius(egui::CornerRadius::from(40.)),
                                |tui| {
                                    tui.colored_label(egui::Color32::BLACK, "ext 1".to_uppercase());
                                },
                            );

                            tui.style(box_style.clone()).bg_add(
                                TuiBackground::new()
                                    .with_border()
                                    .with_border_width(40.)
                                    .with_corner_radius(200),
                                |tui| {
                                    tui.colored_label(egui::Color32::WHITE, "ext 2".to_uppercase());
                                },
                            );
                        },
                    );

                    tui.add(|tui| tui.heading("buttons".to_uppercase()));

                    tui.style(row_style).bg_add(
                        TuiBackground::new().with_background_color(egui::Color32::BLACK),
                        |tui| {
                            let response = tui.style(box_style.clone()).button(|tui| {
                                tui.colored_label(egui::Color32::WHITE, "button".to_uppercase());
                            });

                            if response.clicked() {
                                params.counter += 1;
                            }

                            let label_selectable = if params.selected {
                                "selected"
                            } else {
                                "selectable"
                            };
                            let response =
                                tui.style(box_style.clone())
                                    .selectable(params.selected, |tui| {
                                        tui.colored_label(
                                            egui::Color32::WHITE,
                                            label_selectable.to_uppercase(),
                                        );
                                    });

                            if response.clicked() {
                                params.selected = !params.selected;
                            }

                            let response = tui.style(box_style.clone()).bg_clickable(
                                TuiBackground::new()
                                    .with_background_color(egui::Color32::TRANSPARENT),
                                |tui| {
                                    tui.colored_label(
                                        egui::Color32::WHITE,
                                        "ext clickable transparent".to_uppercase(),
                                    );
                                },
                            );

                            if response.clicked() {
                                params.counter += 1;
                            }

                            let response = tui.style(box_style.clone()).bg_clickable(
                                TuiBackground::new()
                                    .with_border()
                                    .with_border_width_by_response(&|_, _, response| {
                                        if response.hovered() { 10. } else { 2. }
                                    })
                                    .with_corner_radius(20),
                                |tui| {
                                    tui.colored_label(
                                        egui::Color32::WHITE,
                                        "ext clickable".to_uppercase(),
                                    );
                                },
                            );

                            if response.clicked() {
                                params.counter += 1;
                            }

                            let response = tui.style(box_style.clone()).bg_clickable(
                                TuiBackground::new()
                                    .with_background_color(if params.selected {
                                        egui::Color32::GRAY
                                    } else {
                                        egui::Color32::LIGHT_GRAY
                                    })
                                    .with_border_color_by_response(&|_, _, response| {
                                        if response.hovered() {
                                            egui::Color32::DARK_GRAY
                                        } else {
                                            egui::Color32::WHITE
                                        }
                                    })
                                    .with_border_width_by_response(&|_, _, response| {
                                        if response.hovered() { 10. } else { 20. }
                                    })
                                    .with_corner_radius(200),
                                |tui| {
                                    tui.colored_label(
                                        egui::Color32::BLACK,
                                        format!("ext {label_selectable}").to_uppercase(),
                                    );
                                },
                            );

                            if response.clicked() {
                                params.selected = !params.selected;
                            }
                        },
                    );
                    tui.style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Row,
                        gap: length(20f32),
                        ..Default::default()
                    })
                    .add(|tui| {
                        tui.label(format!("clicked: {}x", params.counter).to_uppercase());

                        tui.add(|tui| {
                            tui.label(format!("selected: {}", params.selected).to_uppercase())
                        });
                    });
                })
        });
}

fn holy_grail_demo(ui: &mut egui::Ui, state: &mut State) {
    egui::Window::new("Background holy grail demo")
        .open(&mut state.show_holy_grail_demo)
        .default_width(500.)
        .show(ui.ctx(), |ui| {
            tui(ui, ui.scope_id().with("holy_grail"))
                .reserve_available_space()
                .style(taffy::Style {
                    display: taffy::Display::Grid,
                    size: taffy::Size {
                        width: percent(1f32),
                        height: percent(1f32),
                    },
                    grid_template_columns: vec![length(100f32), fr(1f32), length(100f32)],
                    grid_template_rows: vec![length(100f32), fr(1f32), length(100f32)],
                    ..Default::default()
                })
                .show(|tui| {
                    let style = tui.egui_style_mut();
                    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;

                    let header = taffy::Style {
                        grid_row: style_helpers::line(1),
                        grid_column: span(3),
                        display: taffy::Display::Flex,
                        justify_content: taffy::JustifyContent::CENTER,
                        align_items: taffy::AlignItems::CENTER,

                        ..Default::default()
                    };
                    let left_sidebar = taffy::Style {
                        grid_row: style_helpers::line(2),
                        grid_column: style_helpers::line(1),
                        display: taffy::Display::Flex,
                        justify_content: taffy::JustifyContent::CENTER,
                        align_items: taffy::AlignItems::CENTER,
                        ..Default::default()
                    };
                    let content_area = taffy::Style {
                        grid_row: style_helpers::line(2),
                        grid_column: style_helpers::line(2),
                        display: taffy::Display::Flex,
                        justify_content: taffy::JustifyContent::CENTER,
                        align_items: taffy::AlignItems::CENTER,
                        ..Default::default()
                    };
                    let right_sidebar = taffy::Style {
                        grid_row: style_helpers::line(2),
                        grid_column: style_helpers::line(3),
                        display: taffy::Display::Flex,
                        justify_content: taffy::JustifyContent::CENTER,
                        align_items: taffy::AlignItems::CENTER,
                        ..Default::default()
                    };
                    let footer = taffy::Style {
                        grid_row: style_helpers::line(3),
                        grid_column: span(3),
                        display: taffy::Display::Flex,
                        justify_content: taffy::JustifyContent::CENTER,
                        align_items: taffy::AlignItems::CENTER,
                        ..Default::default()
                    };

                    tui.style(header).bg_add(
                        TuiBackground::new().with_background_color(egui::Color32::WHITE),
                        |tui| {
                            tui.colored_label(egui::Color32::GRAY, "header");
                        },
                    );

                    tui.style(left_sidebar).bg_add(
                        TuiBackground::new().with_background_color(egui::Color32::ORANGE),
                        |tui| {
                            tui.colored_label(egui::Color32::WHITE, "left");
                        },
                    );

                    tui.style(content_area).add(|tui| {
                        tui.colored_label(egui::Color32::WHITE, "content");
                    });

                    tui.style(right_sidebar).bg_add(
                        TuiBackground::new().with_background_color(egui::Color32::MAGENTA),
                        |tui| {
                            tui.colored_label(egui::Color32::WHITE, "right");
                        },
                    );

                    tui.style(footer).bg_add(
                        TuiBackground::new().with_background_color(egui::Color32::GRAY),
                        |tui| {
                            tui.colored_label(egui::Color32::WHITE, "footer");
                        },
                    );
                });
        });
}

/// Native example
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    eframe::run_native(
        "Demo",
        Default::default(),
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

/// Short display name for [`taffy::AlignItems`] / [`taffy::JustifyItems`] (avoids verbose Debug).
fn align_items_name(align: taffy::AlignItems) -> String {
    format!("{:?}", align.keyword)
}

/// Short display name for [`taffy::AlignContent`] / [`taffy::JustifyContent`] (avoids verbose Debug).
fn align_content_name(align: taffy::AlignContent) -> String {
    format!("{:?}", align.keyword)
}
