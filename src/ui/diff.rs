use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

use super::release_notes::release_notes_close_button_rect;
use super::widgets::{
    modal_stack_areas, panel_contrast_fg, render_action_button, render_modal_header,
    render_modal_shell,
};
use crate::app::state::{AppState, DiffPanel, OldLineChange};

const MODAL_WIDTH: u16 = 88;
const MODAL_HEIGHT: u16 = 28;

pub(super) fn render_diff_viewer_overlay(app: &AppState, frame: &mut Frame) {
    super::dim_background(frame, frame.area());

    let Some(inner) = render_modal_shell(frame, frame.area(), MODAL_WIDTH, MODAL_HEIGHT, &app.palette) else {
        return;
    };
    if inner.height < 8 || inner.width < 30 {
        return;
    }

    let stack = modal_stack_areas(inner, 2, 1, 0, 1);
    let header_rows =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas::<2>(stack.header);

    render_modal_header(frame, header_rows[0], "diff viewer", &app.palette);
    render_action_button(
        frame,
        release_notes_close_button_rect(header_rows[0]),
        Some("esc"),
        "close",
        Style::default()
            .fg(panel_contrast_fg(&app.palette))
            .bg(app.palette.accent)
            .add_modifier(Modifier::BOLD),
    );

    let p = &app.palette;
    let d = &app.diff_viewer;
    let body_area = stack.content;

    // --- File tabs (wrapping) ---
    let (_tab_area, panel_area) = render_file_tabs(app, frame, body_area);

    // --- Split panel area ---
    let half = panel_area.width.saturating_sub(1) / 2;
    let right_w = panel_area.width.saturating_sub(1).saturating_sub(half);
    let [left_area, divider, right_area] = Layout::horizontal([
        Constraint::Length(half),
        Constraint::Length(1),
        Constraint::Length(right_w),
    ])
    .areas::<3>(panel_area);

    // Draw vertical divider
    for y in divider.y..divider.y + divider.height {
        let cell = &mut frame.buffer_mut()[(divider.x, y)];
        cell.set_symbol("│");
        cell.set_style(Style::default().fg(p.separator));
    }

    // Render panels
    let left_focused = d.active_panel == DiffPanel::Left;
    let right_focused = d.active_panel == DiffPanel::Right;

    let entry = d.active_file_entry();
    let change_map = entry.map(|e| &e.old_change_map[..]).unwrap_or(&[]);

    render_old_panel(
        frame,
        left_area,
        "old file",
        entry.map(|e| &e.old_content[..]).unwrap_or(&[]),
        change_map,
        d.left_scroll,
        left_focused,
        p,
    );

    render_diff_panel(
        frame,
        right_area,
        "diff",
        entry.map(|e| &e.diff_lines[..]).unwrap_or(&[]),
        d.right_scroll,
        right_focused,
        p,
    );

    // --- Footer help ---
    render_footer(app, frame, stack.footer.unwrap_or_default());
}

/// Render file tabs with wrapping. Returns (tab_area, remaining_panel_area).
fn render_file_tabs<'a>(
    app: &'a AppState,
    frame: &mut Frame,
    body_area: Rect,
) -> (Rect, Rect) {
    let d = &app.diff_viewer;
    if body_area.height < 2 || d.files.is_empty() {
        return (Rect::default(), body_area);
    }

    let p = &app.palette;

    // Collect tab labels
    struct Tab {
        label: String,
        full_w: u16,
    }
    let tabs: Vec<Tab> = d
        .files
        .iter()
        .map(|f| {
            let label = if f.path.is_empty() {
                "changes".to_string()
            } else {
                f.path.rsplit('/').next().unwrap_or(&f.path).to_string()
            };
            Tab {
                full_w: label.chars().count() as u16 + 2,
                label,
            }
        })
        .collect();

    let max_x = body_area.x + body_area.width;


    // First pass: determine how many rows we need
    let mut rows_needed = 1u16;
    let mut x = body_area.x;
    for tab in &tabs {
        if x + tab.full_w > max_x {
            rows_needed = rows_needed.saturating_add(1);
            x = body_area.x;
        }
        x = x.saturating_add(tab.full_w);
    }
    // Clamp to available rows
    let max_rows = body_area.height.saturating_sub(2).min(3).max(1);
    let tab_rows_used = rows_needed.min(max_rows);
    let tab_area_h = tab_rows_used;

    let tab_area = Rect::new(body_area.x, body_area.y, body_area.width, tab_area_h);
    let panel_area = Rect::new(
        body_area.x,
        body_area.y.saturating_add(tab_area_h),
        body_area.width,
        body_area.height.saturating_sub(tab_area_h),
    );

    // If some files don't fit, show "+N" at the end of the last row
    let max_visible_tabs = if rows_needed > max_rows {
        // Count visible tabs
        let mut count = 0usize;
        let mut rx = body_area.x;
        let mut r = 0u16;
        for tab in &tabs {
            if rx + tab.full_w > max_x {
                r = r.saturating_add(1);
                if r >= max_rows {
                    break;
                }
                rx = body_area.x;
            }
            if r >= max_rows {
                break;
            }
            rx = rx.saturating_add(tab.full_w);
            count = count.saturating_add(1);
        }
        count
    } else {
        tabs.len()
    };
    let hidden_count = tabs.len().saturating_sub(max_visible_tabs);

    // Second pass: render visible tabs
    let mut row = 0u16;
    let mut cx = tab_area.x;
    let mut cy = tab_area.y;

    for (i, tab) in tabs.iter().enumerate() {
        if i >= max_visible_tabs {
            break;
        }

        let next_x = cx.saturating_add(tab.full_w);

        // Check if we need to wrap
        if next_x > max_x {
            row = row.saturating_add(1);
            if row >= max_rows {
                break;
            }
            cx = tab_area.x;
            cy = cy.saturating_add(1);
        }

        // Recalculate after possible wrap
        let this_x = cx;
        let this_w = tab.full_w;
        cx = cx.saturating_add(tab.full_w);

        let remaining = max_x.saturating_sub(this_x);
        let display = if this_w > remaining {
            let max_chars = (remaining.max(4) - 1).min(tab.label.len() as u16) as usize;
            format!(
                " {}…",
                &tab.label[..max_chars.saturating_sub(2).min(tab.label.len())]
            )
        } else {
            format!(" {} ", tab.label)
        };

        let is_active = i == d.active_file;

        let style = if is_active {
            Style::default()
                .fg(p.text)
                .bg(p.surface0)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.overlay0)
        };

        let disp_w = display.chars().count() as u16;
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(display, style)])),
            Rect::new(this_x, cy, disp_w.min(remaining), 1),
        );
    }

    // Render "+N remaining" if tabs were cut
    if hidden_count > 0 {
        let suffix = format!(" +{}", hidden_count);
        let suffix_w = suffix.chars().count() as u16;
        let last_x = cx.saturating_add(2);
        if last_x + suffix_w <= max_x {
            frame.render_widget(
                Paragraph::new(Line::from(vec![Span::styled(
                    suffix,
                    Style::default().fg(p.overlay0).add_modifier(Modifier::ITALIC),
                )])),
                Rect::new(last_x, cy, suffix_w, 1),
            );
        } else if row < max_rows.saturating_sub(1) {
            let next_y = cy.saturating_add(1);
            frame.render_widget(
                Paragraph::new(Line::from(vec![Span::styled(
                    suffix,
                    Style::default().fg(p.overlay0).add_modifier(Modifier::ITALIC),
                )])),
                Rect::new(body_area.x, next_y, suffix_w, 1),
            );
        }
    }

    (tab_area, panel_area)
}

/// Render the old file panel with change highlighting.
fn render_old_panel(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    lines: &[String],
    change_map: &[OldLineChange],
    scroll: usize,
    focused: bool,
    p: &crate::app::state::Palette,
) {
    if area.width < 4 || area.height < 2 {
        return;
    }

    let title_style = if focused {
        Style::default()
            .fg(p.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.overlay0)
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            format!(" {} ", title),
            title_style,
        )])),
        Rect::new(area.x, area.y, area.width.min(80), 1),
    );

    let content_area = Rect::new(
        area.x,
        area.y.saturating_add(1),
        area.width,
        area.height.saturating_sub(1),
    );
    if content_area.height == 0 {
        return;
    }

    // Style each line based on its change status
    let styled: Vec<Line<'_>> = lines
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            let status = change_map.get(idx).copied().unwrap_or(OldLineChange::None);
            let style = match status {
                OldLineChange::Removed => Style::default()
                    .fg(Color::Red)
                    .bg(p.surface0)
                    .add_modifier(Modifier::DIM),
                OldLineChange::Context => Style::default().fg(Color::Yellow),
                OldLineChange::None => Style::default().fg(p.text),
            };
            let prefix = match status {
                OldLineChange::Removed => "▌ ",
                OldLineChange::Context => "· ",
                OldLineChange::None => "  ",
            };
            Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::styled(line.as_str(), style),
            ])
        })
        .collect();

    // Scrollbar
    let metrics = crate::pane::ScrollMetrics {
        offset_from_bottom: lines
            .len()
            .max(1)
            .saturating_sub(1)
            .saturating_sub(scroll),
        max_offset_from_bottom: lines.len().max(1).saturating_sub(1),
        viewport_rows: content_area.height.max(1) as usize,
    };

    let text_area = if metrics.max_offset_from_bottom > 0 {
        let track_x = content_area.x + content_area.width.saturating_sub(1);
        let track = Rect::new(track_x, content_area.y, 1, content_area.height);
        super::scrollbar::render_scrollbar(
            frame,
            metrics,
            track,
            p.overlay0,
            p.overlay1,
            "▐",
        );
        Rect::new(
            content_area.x,
            content_area.y,
            content_area.width.saturating_sub(1),
            content_area.height,
        )
    } else {
        content_area
    };

    let body = Paragraph::new(styled)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(body, text_area);
}

/// Render the diff panel with syntax-colored diff lines.
fn render_diff_panel(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    lines: &[String],
    scroll: usize,
    focused: bool,
    p: &crate::app::state::Palette,
) {
    if area.width < 4 || area.height < 2 {
        return;
    }

    let title_style = if focused {
        Style::default()
            .fg(p.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.overlay0)
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            format!(" {} ", title),
            title_style,
        )])),
        Rect::new(area.x, area.y, area.width.min(80), 1),
    );

    let content_area = Rect::new(
        area.x,
        area.y.saturating_add(1),
        area.width,
        area.height.saturating_sub(1),
    );
    if content_area.height == 0 {
        return;
    }

    // Style diff lines with prefix indicators
    let styled: Vec<Line<'_>> = lines
        .iter()
        .map(|line| {
            let (color, prefix, text) = if line.starts_with('+') && !line.starts_with("+++") {
                (Color::Green, "▌", &line[1..])
            } else if line.starts_with('-') && !line.starts_with("---") {
                (Color::Red, "▌", &line[1..])
            } else if line.starts_with("@@") {
                (Color::Cyan, "·", line.as_str())
            } else if line.starts_with("diff --git")
                || line.starts_with("index ")
                || line.starts_with("---")
                || line.starts_with("+++")
            {
                (p.overlay0, " ", line.as_str())
            } else {
                (p.text, " ", line.as_str())
            };
            let prefix_style = Style::default().fg(
                if color == Color::Green || color == Color::Red {
                    color
                } else {
                    Color::DarkGray
                },
            );
            let text_style = if color == Color::Green || color == Color::Red || color == Color::Cyan
            {
                Style::default().fg(color)
            } else {
                Style::default().fg(color)
            };
            let bg = if color == Color::Green {
                Some(Color::Green)
            } else if color == Color::Red {
                Some(Color::Red)
            } else {
                None
            };
            let text_style = if let Some(bg_color) = bg {
                text_style.bg(bg_color).add_modifier(Modifier::DIM)
            } else {
                text_style
            };
            Line::from(vec![
                Span::styled(prefix, prefix_style),
                Span::styled(text, text_style),
            ])
        })
        .collect();

    // Scrollbar
    let metrics = crate::pane::ScrollMetrics {
        offset_from_bottom: lines
            .len()
            .max(1)
            .saturating_sub(1)
            .saturating_sub(scroll),
        max_offset_from_bottom: lines.len().max(1).saturating_sub(1),
        viewport_rows: content_area.height.max(1) as usize,
    };

    let text_area = if metrics.max_offset_from_bottom > 0 {
        let track_x = content_area.x + content_area.width.saturating_sub(1);
        let track = Rect::new(track_x, content_area.y, 1, content_area.height);
        super::scrollbar::render_scrollbar(
            frame,
            metrics,
            track,
            p.overlay0,
            p.overlay1,
            "▐",
        );
        Rect::new(
            content_area.x,
            content_area.y,
            content_area.width.saturating_sub(1),
            content_area.height,
        )
    } else {
        content_area
    };

    let body = Paragraph::new(styled)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(body, text_area);
}

fn render_footer(app: &AppState, frame: &mut Frame, area: Rect) {
    if area.width < 10 {
        return;
    }

    let p = &app.palette;
    let d = &app.diff_viewer;
    let file_count = d.files.len();
    let file_label = if file_count > 0 {
        let entry = &d.files[d.active_file];
        let name = entry.path.rsplit('/').next().unwrap_or(&entry.path);
        format!("  {}  ({}/{})", name, d.active_file + 1, file_count)
    } else {
        String::new()
    };

    let panel_label = match d.active_panel {
        DiffPanel::Left => "  [old file]",
        DiffPanel::Right => "  [diff]",
    };

    let mut spans = Vec::new();
    if !file_label.is_empty() {
        spans.push(Span::styled(
            file_label,
            Style::default().fg(p.subtext0),
        ));
        spans.push(Span::styled(" ", Style::default().fg(p.overlay0)));
    }
    spans.push(Span::styled(
        panel_label,
        Style::default()
            .fg(p.accent)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("  ", Style::default()));
    spans.extend(vec![
        Span::styled("scroll", Style::default().fg(p.overlay0)),
        Span::styled(" j/k ↑↓ ", Style::default().fg(p.text)),
        Span::styled("  ·  ", Style::default().fg(p.overlay0)),
        Span::styled("panel", Style::default().fg(p.overlay0)),
        Span::styled(" h/l ", Style::default().fg(p.text)),
        Span::styled("  ·  ", Style::default().fg(p.overlay0)),
        Span::styled("file", Style::default().fg(p.overlay0)),
        Span::styled(" n/p tab ", Style::default().fg(p.text)),
        Span::styled("  ·  ", Style::default().fg(p.overlay0)),
        Span::styled("close", Style::default().fg(p.overlay0)),
        Span::styled(" esc ", Style::default().fg(p.text)),
    ]);

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
