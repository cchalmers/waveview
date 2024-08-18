use eframe::egui;
use egui::*;
pub mod paint_ticks;
pub mod time_view;
pub mod time_ranges_ui;

use time_view::TimeView;

/// A panel that shows entity names to the left, time on the top.
///
/// This includes the timeline controls and streams view.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Debug, Default)]
pub struct Timeline {
    /// Width of the entity name columns previous frame.
    prev_col_width: f32,

    /// The right side of the entity name column; updated during its painting.
    #[serde(skip)]
    next_col_right: f32,

    /// The time axis view, regenerated each frame.
    #[serde(skip)]
    time_ranges_ui: time_ranges_ui::TimeRangesUi,

    time_control: time_view::TimeControl,

    pub first: bool,

    // /// Ui elements for controlling time.
    // time_control_ui: TimeControlUi,
}


impl Timeline {
    pub fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        let top_bar_height = 20.0;
        ui.vertical(|ui| {
            // Add back the margin we removed from the panel:
            let mut top_row_frame = egui::Frame::default();
            // top_row_frame.inner_margin.right = margin.right;
            // top_row_frame.inner_margin.bottom = margin.bottom;
            let top_row_rect = top_row_frame
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().interact_size = Vec2::splat(top_bar_height);
                        ui.visuals_mut().button_frame = true;
                        self.top_row_ui(ctx, ui);
                    });
                })
                .response
                .rect;

            // Draw separator between top bar and the rest:
            ui.painter().hline(
                0.0..=top_row_rect.right(),
                top_row_rect.bottom(),
                ui.visuals().widgets.noninteractive.bg_stroke,
            );

            ui.spacing_mut().scroll.bar_outer_margin = 4.0; // needed, because we have no panel margin on the right side.

            // Add extra margin on the left which was intentionally missing on the controls.
            let mut streams_frame = egui::Frame::default();
            // streams_frame.inner_margin.left = margin.left;
            streams_frame.show(ui, |ui| {
                self.expanded_ui(ctx, ui);
            });
        });
    }

    fn top_row_ui(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("top row");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("?");
            });
        });
    }

    fn expanded_ui(&mut self, ctx: &Context, ui: &mut Ui) {
        self.next_col_right = ui.min_rect().left(); // next_col_right will expand during the call

        let time_x_left =
            (ui.min_rect().left() + self.prev_col_width + ui.spacing().item_spacing.x)
                .at_most(ui.max_rect().right() - 100.0)
                .at_least(100.); // cover the empty recording case

        // Where the time will be shown.
        let time_bg_x_range = Rangef::new(time_x_left, ui.max_rect().right());
        let time_fg_x_range = {
            // Painting to the right of the scroll bar (if any) looks bad:
            let right = ui.max_rect().right() - ui.spacing_mut().scroll.bar_outer_margin;
            debug_assert!(time_x_left < right);
            Rangef::new(time_x_left, right)
        };

        let side_margin = 26.0; // chosen so that the scroll bar looks approximately centered in the default gap
        // self.time_ranges_ui = initialize_time_ranges_ui(
        //     entity_db,
        //     time_ctrl,
        //     Rangef::new(
        //         time_fg_x_range.min + side_margin,
        //         time_fg_x_range.max - side_margin,
        //     ),
        //     time_ctrl.time_view(),
        // );
        let full_y_range = Rangef::new(ui.min_rect().bottom(), ui.max_rect().bottom());

        let timeline_rect = {
            let top = ui.min_rect().bottom();

            let size = egui::vec2(self.prev_col_width, 28.0);
            ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.set_min_size(size);
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                ui.add_space(4.0); // hack to vertically center the text
                ui.strong("Streams");
                // if self.source == TimePanelSource::Blueprint {
                //     ui.strong("Blueprint Streams");
                // } else {
                //     ui.strong("Streams");
                // }
            })
            .response
            .on_hover_text(
                "A hierarchical view of the paths used during logging.\n\
                        \n\
                        On the right you can see when there was a log event for a stream.",
            );

            let bottom = ui.min_rect().bottom();
            Rect::from_x_y_ranges(time_fg_x_range, top..=bottom)
        };

        let streams_rect = Rect::from_x_y_ranges(
            time_fg_x_range,
            timeline_rect.bottom()..=ui.max_rect().bottom(),
        );

        let time_range_f = re_log_types::ResolvedTimeRangeF::new(
            -12.0,
            120.0,
            // 1e9,
        );

        let time_range = re_log_types::ResolvedTimeRange::new(
            0i64,
            100i64,
            // 1e9,
        );

        // static FIRST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

        // if FIRST.load(std::sync::atomic::Ordering::Relaxed) {
        //     self.time_ranges_ui = time_ranges_ui::TimeRangesUi::new(
        //         time_fg_x_range,
        //         TimeView {
        //             min: 0.into(),
        //             time_spanned: 1000.into(),
        //         },
        //         &[time_range],
        //     );
        //     FIRST.store(false, std::sync::atomic::Ordering::Relaxed);

        //     eprintln!("there are {} segments", self.time_ranges_ui.segments.len());
        // }

            self.time_ranges_ui = time_ranges_ui::TimeRangesUi::new(
                time_fg_x_range,
                self.time_control.time_view().unwrap_or(TimeView {
                    min: 0.into(),
                    time_spanned: 500.into(),
                }),
                &[time_range],
            );

        // includes the timeline and streams areas.
        let time_bg_area_rect = Rect::from_x_y_ranges(time_bg_x_range, full_y_range);
        let time_fg_area_rect = Rect::from_x_y_ranges(time_fg_x_range, full_y_range);
        let time_bg_area_painter = ui.painter().with_clip_rect(time_bg_area_rect);
        let time_area_painter = ui.painter().with_clip_rect(time_fg_area_rect);

        // if let Some(highlighted_range) = time_ctrl.highlighted_range {
        //     paint_range_highlight(
        //         highlighted_range,
        //         &self.time_ranges_ui,
        //         ui.painter(),
        //         time_fg_area_rect,
        //     );
        // }

        ui.painter().hline(
            0.0..=ui.max_rect().right(),
            timeline_rect.bottom(),
            ui.visuals().widgets.noninteractive.bg_stroke,
        );

    // mut x_range: RangeInclusive<f64>,
    // mut time_range: ResolvedTimeRangeF,

        let x_range = time_bg_area_rect.x_range();
        // let x_range =
        //     Rangef::new(
        //         time_fg_x_range.min + side_margin,
        //         time_fg_x_range.max - side_margin,
        //     );

        paint_ticks::paint_time_ranges_and_ticks(
            &self.time_ranges_ui,
            // time_fg_x_range.min as f64 + side_margin..=time_fg_x_range.max as f64 - side_margin,
            // time_fg_x_range.min as f64 ..=time_fg_x_range.max as f64,
            // time_range_f,
            ui,
            &time_area_painter,
            timeline_rect.top()..=timeline_rect.bottom(),
            // time_ctrl.time_type(),
            re_log_types::TimeType::Sequence,
            re_log_types::TimeZone::Utc,
        );

        // paint_ticks::paint_time_ranges_and_ticks_range(
        //     // &self.time_ranges_ui,
        //     // time_fg_x_range.min as f64 + side_margin..=time_fg_x_range.max as f64 - side_margin,
        //     time_fg_x_range.min as f64 ..=time_fg_x_range.max as f64,
        //     time_range_f,
        //     ui,
        //     &time_area_painter,
        //     timeline_rect.top()..=timeline_rect.bottom(),
        //     // time_ctrl.time_type(),
        //     re_log_types::TimeType::Sequence,
        //     re_log_types::TimeZone::Utc,
        // );

        // paint_time_ranges_gaps(
        //     &self.time_ranges_ui,
        //     ui,
        //     &time_bg_area_painter,
        //     full_y_range,
        // );
        // time_selection_ui::loop_selection_ui(
        //     time_ctrl,
        //     &self.time_ranges_ui,
        //     ui,
        //     &time_bg_area_painter,
        //     &timeline_rect,
        // );
        let time_area_response = interact_with_streams_rect(
            &self.time_ranges_ui,
            &mut self.time_control,
            ui,
            &time_bg_area_rect,
            &streams_rect,
        );

        // Don't draw on top of the time ticks
        let lower_time_area_painter = ui.painter().with_clip_rect(Rect::from_x_y_ranges(
            time_fg_x_range,
            ui.min_rect().bottom()..=ui.max_rect().bottom(),
        ));

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            // We turn off `drag_to_scroll` so that the `ScrollArea` don't steal input from
            // the earlier `interact_with_time_area`.
            // We implement drag-to-scroll manually instead!
            .drag_to_scroll(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 10.0; // no spacing needed for ListItems

                // if time_area_response.dragged_by(PointerButton::Primary) {
                //     ui.scroll_with_delta(Vec2::Y * time_area_response.drag_delta().y);
                // }

                for i in 0..50 {
                    let resp = ui.label(&format!("Hello, world! {}", i));
                    highlight_timeline_row(ui, resp.hovered, false, ui.painter(), &resp.rect);
                }

                // // Show "/" on top?
                // let show_root = true;

                // if show_root {
                //     self.show_tree(
                //         ctx,
                //         viewport_blueprint,
                //         entity_db,
                //         time_ctrl,
                //         time_area_response,
                //         time_area_painter,
                //         None,
                //         entity_db.tree(),
                //         ui,
                //         "/",
                //     );
                // } else {
                //     self.show_children(
                //         ctx,
                //         viewport_blueprint,
                //         entity_db,
                //         time_ctrl,
                //         time_area_response,
                //         time_area_painter,
                //         entity_db.tree(),
                //         ui,
                //     );
                // }
            });
        // All the entity rows and their data density graphs
        // ui.full_span_scope(0.0..=time_x_left, |ui| {

            // list_item::list_item_scope(ui, "streams_tree", |ui| {
            //     self.tree_ui(
            //         ctx,
            //         viewport_blueprint,
            //         entity_db,
            //         time_ctrl,
            //         &time_area_response,
            //         &lower_time_area_painter,
            //         ui,
            //     );
            // });
        // });

        {
            // Paint a shadow between the stream names on the left
            // and the data on the right:
            let shadow_width = 6.0;

            // In the design the shadow starts under the time markers.
            //let shadow_y_start =
            //    timeline_rect.bottom() + ui.visuals().widgets.noninteractive.bg_stroke.width;
            // This looks great but only if there are still time markers.
            // When they move to the right (or have a cut) one expects the shadow to go all the way up.
            // But that's quite complicated so let's have the shadow all the way
            let shadow_y_start = full_y_range.min;

            let shadow_y_end = full_y_range.max;
            let rect = egui::Rect::from_x_y_ranges(
                time_x_left..=(time_x_left + shadow_width),
                shadow_y_start..=shadow_y_end,
            );
            draw_shadow_line(ui, rect, egui::Direction::LeftToRight);
        }

        // Put time-marker on top and last, so that you can always drag it
        // time_marker_ui(
        //     &self.time_ranges_ui,
        //     time_ctrl,
        //     ui,
        //     &time_area_painter,
        //     &timeline_rect,
        // );

        // self.time_ranges_ui.snap_time_control(time_ctrl);

        // remember where to show the time for next frame:
        self.prev_col_width = self.next_col_right - ui.min_rect().left();
    }
}

/// Draw the hovered/selected highlight background for a timeline row.
fn highlight_timeline_row(
    ui: &Ui,
    item_hovered: bool,
    item_selected: bool,
    painter: &Painter,
    row_rect: &Rect,
) {
    let bg_color = if item_selected {
        Some(ui.visuals().selection.bg_fill.gamma_multiply(0.4))
    } else if item_hovered {
        Some(
            ui.visuals()
                .widgets
                .hovered
                .weak_bg_fill
                .gamma_multiply(0.3),
        )
    } else {
        None
    };
    if let Some(bg_color) = bg_color {
        painter.rect_filled(*row_rect, egui::Rounding::ZERO, bg_color);
    }
}

// fn initialize_time_ranges_ui(
//     entity_db: &re_entity_db::EntityDb,
//     time_ctrl: &time_view::TimeControl,
//     time_x_range: Rangef,
//     mut time_view: Option<TimeView>,
// ) -> TimeRangesUi {

//     let mut time_range = Vec::new();

//     if let Some(times) = entity_db.time_histogram(time_ctrl.timeline()) {
//         // NOTE: `times` can be empty if a GC wiped everything.
//         if !times.is_empty() {
//             let timeline_axis = TimelineAxis::new(time_ctrl.time_type(), times);
//             time_view = time_view.or_else(|| Some(view_everything(&time_x_range, &timeline_axis)));
//             time_range.extend(timeline_axis.ranges);
//         }
//     }

//     TimeRangesUi::new(
//         time_x_range,
//         time_view.unwrap_or(TimeView {
//             min: TimeReal::from(0),
//             time_spanned: 1.0,
//         }),
//         &time_range,
//     )
// }

/// Returns a scroll delta
#[must_use]
fn interact_with_streams_rect(
    time_ranges_ui: &time_ranges_ui::TimeRangesUi,
    time_ctrl: &mut time_view::TimeControl,
    ui: &egui::Ui,
    full_rect: &Rect,
    streams_rect: &Rect,
) -> egui::Response {
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());

    let mut delta_x = 0.0;
    let mut zoom_factor = 1.0;

    // Check for zoom/pan inputs (via e.g. horizontal scrolling) on the entire
    // time area rectangle, including the timeline rect.
    let full_rect_hovered =
        pointer_pos.map_or(false, |pointer_pos| full_rect.contains(pointer_pos));
    if full_rect_hovered {
        ui.input(|input| {
            delta_x += input.smooth_scroll_delta.x;
            zoom_factor *= input.zoom_delta_2d().x;
        });
    }

    // We only check for drags in the streams rect,
    // because drags in the timeline rect should move the time
    // (or create loop sections).
    let response = ui.interact(
        *streams_rect,
        ui.id().with("time_area_interact"),
        egui::Sense::click_and_drag(),
    );
    if response.dragged_by(PointerButton::Primary) {
        delta_x += response.drag_delta().x;
        ui.ctx().set_cursor_icon(CursorIcon::AllScroll);
    }
    if response.dragged_by(PointerButton::Secondary) {
        zoom_factor *= (response.drag_delta().y * 0.01).exp();
    }

    if delta_x != 0.0 {
        if let Some(new_view_range) = time_ranges_ui.pan(-delta_x) {
            time_ctrl.set_time_view(new_view_range);
        }
    }

    if zoom_factor != 1.0 {
        if let Some(pointer_pos) = pointer_pos {
            if let Some(new_view_range) = time_ranges_ui.zoom_at(pointer_pos.x, zoom_factor) {
                time_ctrl.set_time_view(new_view_range);
            }
        }
    }

    if response.double_clicked() {
        time_ctrl.reset_time_view();
    }

    response
}

/// Draws a shadow into the given rect with the shadow direction given from dark to light
fn draw_shadow_line(ui: &Ui, rect: Rect, direction: egui::Direction) {
    let color_dark = egui::Color32::from_black_alpha(77); // design_tokens().shadow_gradient_dark_start;
    let color_bright = Color32::TRANSPARENT;

    let (left_top, right_top, left_bottom, right_bottom) = match direction {
        egui::Direction::RightToLeft => (color_bright, color_dark, color_bright, color_dark),
        egui::Direction::LeftToRight => (color_dark, color_bright, color_dark, color_bright),
        egui::Direction::BottomUp => (color_bright, color_bright, color_dark, color_dark),
        egui::Direction::TopDown => (color_dark, color_dark, color_bright, color_bright),
    };

    use egui::epaint::Vertex;
    let shadow = egui::Mesh {
        indices: vec![0, 1, 2, 2, 1, 3],
        vertices: vec![
            Vertex {
                pos: rect.left_top(),
                uv: egui::epaint::WHITE_UV,
                color: left_top,
            },
            Vertex {
                pos: rect.right_top(),
                uv: egui::epaint::WHITE_UV,
                color: right_top,
            },
            Vertex {
                pos: rect.left_bottom(),
                uv: egui::epaint::WHITE_UV,
                color: left_bottom,
            },
            Vertex {
                pos: rect.right_bottom(),
                uv: egui::epaint::WHITE_UV,
                color: right_bottom,
            },
        ],
        texture_id: Default::default(),
    };
    ui.painter().add(shadow);
}
