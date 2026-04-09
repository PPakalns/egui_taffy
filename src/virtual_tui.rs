use taffy::{
    CompactLength,
    prelude::{auto, length},
};

use crate::{Tui, TuiBuilderLogic, TuiId, tid};

/// Required parameters to correctly draw grid with virtual rows
pub struct VirtualGridRowHelperParams {
    /// Header row count that needs to be skipped in the grid
    pub header_row_count: u16,
    /// Data row count in the grid excluding any header rows
    pub row_count: usize,
}

/// Helper to draw grid with virtual rows
pub struct VirtualGridRowHelper;

/// Information about grid row that needs to be drawn
pub struct VirtualGridRow {
    /// Index of data from 0..row_count
    pub idx: usize,
    /// Row position in the grid
    ///
    /// Use [`GridRow::set_grid_row`] to retrieve closure that will set the style.
    pub grid_row: u16,
}

impl VirtualGridRow {
    /// Retrieve closure that can be used in `tui.mut_style(_)` to set grid_row parameter.
    #[inline]
    pub fn grid_row_setter(&self) -> impl Fn(&mut taffy::Style) {
        let grid_row = self.grid_row;
        move |style: &mut taffy::Style| {
            style.grid_row = taffy::style_helpers::line(grid_row as i16);
        }
    }

    /// Retrieve closure that can be used to generate unique ids for elements in the row
    #[inline]
    pub fn id_gen(&self) -> impl FnMut() -> TuiId {
        let idx = self.idx;
        let mut col_idx = 0;
        move || {
            col_idx += 1;
            tid(("cell", idx, col_idx))
        }
    }
}

const fn round_up_to_pow2(value: usize, pow2: u8) -> usize {
    value.saturating_add((1 << pow2) - 1) & !((1 << pow2) - 1)
}

const fn round_down_to_pow2(value: usize, pow2: u8) -> usize {
    value & !((1 << pow2) - 1)
}

impl VirtualGridRowHelper {
    /// Show virtual grid rows.
    ///
    /// Closure receives information about grid row that needs to be drawn.
    /// All virtual rows should have equal heaight. One row will be used to estimate height of all rows.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    pub fn show<F>(params: VirtualGridRowHelperParams, tui: &mut Tui, mut draw_line: F)
    where
        F: FnMut(&mut Tui, VirtualGridRow),
    {
        let VirtualGridRowHelperParams {
            row_count,
            header_row_count,
        } = params;

        if row_count == 0 {
            return;
        }

        // 1-based grid_row counter
        let mut grid_row = header_row_count + 1;

        // Draw first row for reference
        draw_line(tui, VirtualGridRow { idx: 0, grid_row });

        if row_count == 1 {
            return;
        }

        let node_id = tui.current_node();

        let min_location = (tui.taffy_container().full_container_with(false).min
            - tui.current_viewport_content().min)
            .y;

        let (top_offset, row_height, gap) = {
            let state = tui.taffy_state();

            let style = state.taffy_tree().style(node_id).unwrap();

            let gap = {
                let raw = style.gap.height.into_raw();
                match raw.tag() {
                    CompactLength::LENGTH_TAG => raw.value(),
                    CompactLength::PERCENT_TAG => 0.,
                    _ => 0.,
                }
            };

            let mut top_offset = match style.overflow.y {
                taffy::Overflow::Visible | taffy::Overflow::Clip | taffy::Overflow::Hidden => {
                    min_location
                }
                taffy::Overflow::Scroll => 0.,
            };
            let layout_detailed_info = state.taffy_tree().detailed_layout_info(node_id);

            match layout_detailed_info {
                taffy::DetailedLayoutInfo::Grid(detailed_grid_info) => {
                    // Calculate header offset
                    for idx in 0..((grid_row - 1) as usize) {
                        if let Some(row_size) = detailed_grid_info.rows.sizes.get(idx) {
                            top_offset += row_size;
                        }

                        if let Some(gutter) = detailed_grid_info.rows.gutters.get(idx) {
                            top_offset += gutter;
                        }
                    }

                    let row_height = detailed_grid_info
                        .rows
                        .sizes
                        .get((grid_row - 1) as usize)
                        .copied()
                        .unwrap_or(20.);

                    (top_offset, row_height, gap)
                }
                taffy::DetailedLayoutInfo::None => (top_offset, 20., gap),
            }
        };

        let full_row_height = row_height + gap;

        let scroll_offset = -(tui.last_scroll_offset.y + top_offset);
        let visible_rect_size = tui.current_viewport().size().y;

        // Round to power of 2 numbers to reduce frequency of taffy layout recalculation
        // TODO: Maybe store interval in memory?
        let pow2 = 3; // 2^3 = 8

        // How many items should be drawn at top and bottom
        let buffer = 4.;

        let visible_from = round_down_to_pow2(
            ((scroll_offset / full_row_height).floor() - buffer).max(0.) as usize,
            pow2,
        )
        .clamp(1, row_count);

        let visible_to = round_up_to_pow2(
            (((scroll_offset + visible_rect_size) / full_row_height).ceil() + buffer).max(0.)
                as usize,
            pow2,
        )
        .clamp(visible_from, row_count);

        println!(
            "{} {} {} | {} {} {} {} {}",
            visible_from,
            visible_to,
            row_count,
            row_height,
            gap,
            scroll_offset,
            top_offset,
            visible_rect_size
        );

        if visible_from > 1 {
            // Draw empty cell from 1..next_visible_from

            let row_count_to_hide = visible_from - 1;
            let height = (row_count_to_hide as f32) * full_row_height - gap;

            grid_row += 1;

            let size = taffy::Size {
                width: length(0.),
                height: length(height),
            };

            tui.id("top_virtual")
                .style(taffy::Style {
                    min_size: size,
                    size,
                    max_size: size,
                    grid_row: taffy::style_helpers::line(grid_row as i16),
                    ..Default::default()
                })
                .add_empty();
        }

        if visible_from < visible_to {
            for row_idx in visible_from..visible_to {
                grid_row += 1;

                draw_line(
                    tui,
                    VirtualGridRow {
                        idx: row_idx,
                        grid_row,
                    },
                );
            }
        }

        if visible_to < row_count {
            // Draw empty cell from visible_to..row_count

            let row_count_to_hide = row_count - visible_to;
            let height = (row_count_to_hide as f32) * full_row_height - gap;

            grid_row += 1;

            let size = taffy::Size {
                width: auto(),
                height: length(height),
            };

            tui.id("bottom_virtual")
                .style(taffy::Style {
                    min_size: size,
                    size,
                    max_size: size,
                    grid_row: taffy::style_helpers::line(grid_row as i16),
                    ..Default::default()
                })
                .add_empty();
        }
    }
}
