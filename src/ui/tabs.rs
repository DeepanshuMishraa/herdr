use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::widgets::panel_contrast_fg;
use crate::app::{state::Mode, AppState};
use crate::terminal::TerminalRuntimeRegistry;

const MIN_TAB_WIDTH: u16 = 8;
const NEW_TAB_WIDTH: u16 = 3;
const TAB_SCROLL_BUTTON_WIDTH: u16 = 3;

#[derive(Debug, Clone, Default)]
pub(crate) struct TabBarView {
    pub scroll: usize,
    pub tab_hit_areas: Vec<Rect>,
    pub scroll_left_hit_area: Rect,
    pub scroll_right_hit_area: Rect,
    pub new_tab_hit_area: Rect,
}

fn tab_title_with_zoom(
    app: &AppState,
    ws: &crate::workspace::Workspace,
    tab_idx: usize,
    terminal_runtimes: &TerminalRuntimeRegistry,
) -> String {
    let mut name = if let Some(tab) = ws.tabs.get(tab_idx) {
        if let Some(ref custom_name) = tab.custom_name {
            custom_name.clone()
        } else {
            let pane_id = tab.layout.focused();
            if let Some(pane) = tab.panes.get(&pane_id) {
                let terminal_id = &pane.attached_terminal_id;
                let mut detected_name = None;

                // 1. Try to query the runtime for live process/OSC information
                if let Some(runtime) = terminal_runtimes.get(terminal_id) {
                    let mut is_shell_only = true;
                    let mut pg_name = None;

                    if let Some(child_pid) = runtime.child_pid() {
                        if let Some(job) = crate::detect::foreground_job(child_pid) {
                            let non_shell = job.processes.iter().find(|p| {
                                !matches!(
                                    p.name.as_str(),
                                    "zsh" | "bash" | "sh" | "fish" | "csh" | "ksh" | "tcsh"
                                )
                            });
                            if let Some(proc) = non_shell {
                                is_shell_only = false;
                                pg_name = Some(proc.name.clone());
                            } else if let Some(first_proc) = job.processes.first() {
                                pg_name = Some(first_proc.name.clone());
                            }
                        }
                    }

                    if is_shell_only {
                        detected_name = Some(pg_name.unwrap_or_else(|| "zsh".to_string()));
                    } else {
                        let osc_title = runtime.agent_osc_title();
                        if !osc_title.is_empty()
                            && !(osc_title.contains('@') && osc_title.contains(':'))
                        {
                            detected_name = Some(osc_title);
                        } else {
                            detected_name = pg_name;
                        }
                    }
                }

                // 2. Fallback to terminal border_label
                if detected_name.is_none() {
                    if let Some(terminal) = app.terminals.get(terminal_id) {
                        if let Some(label) =
                            terminal.border_label(app.show_agent_labels_on_pane_borders)
                        {
                            if !(label.contains('@') && label.contains(':')) {
                                detected_name = Some(label);
                            }
                        }
                    }
                }

                // 3. Fallback to launch_argv
                if detected_name.is_none() {
                    if let Some(terminal) = app.terminals.get(terminal_id) {
                        if let Some(ref argv) = terminal.launch_argv {
                            if let Some(first_arg) = argv.first() {
                                if let Some(bin_name) = std::path::Path::new(first_arg).file_name()
                                {
                                    detected_name = Some(bin_name.to_string_lossy().into_owned());
                                } else {
                                    detected_name = Some(first_arg.clone());
                                }
                            }
                        }
                    }
                }

                detected_name.unwrap_or_else(|| "zsh".to_string())
            } else {
                "zsh".to_string()
            }
        }
    } else {
        "zsh".to_string()
    };

    if ws.tabs.get(tab_idx).is_some_and(|tab| tab.zoomed) {
        name.push_str(" Z");
    }
    name
}

fn tab_width(
    app: &AppState,
    ws: &crate::workspace::Workspace,
    tab_idx: usize,
    terminal_runtimes: &TerminalRuntimeRegistry,
) -> u16 {
    let name_len = tab_title_with_zoom(app, ws, tab_idx, terminal_runtimes)
        .chars()
        .count();
    let num_len = (tab_idx + 1).to_string().chars().count();
    ((num_len + name_len + 4) as u16).max(MIN_TAB_WIDTH)
}

fn layout_tab_hit_areas(
    app: &AppState,
    ws: &crate::workspace::Workspace,
    area: Rect,
    scroll: usize,
    terminal_runtimes: &TerminalRuntimeRegistry,
) -> Vec<Rect> {
    let mut rects = vec![Rect::default(); ws.tabs.len()];
    if area.width == 0 || area.height == 0 {
        return rects;
    }

    let mut x = area.x;
    let right = area.x + area.width;
    for (idx, rect) in rects.iter_mut().enumerate().skip(scroll) {
        if x >= right {
            break;
        }
        let desired = tab_width(app, ws, idx, terminal_runtimes);
        let remaining = right.saturating_sub(x);
        let width = desired.min(remaining).max(1);
        *rect = Rect::new(x, area.y, width, 1);
        x = x.saturating_add(width + 1);
    }
    rects
}

fn centered_tab_scroll(
    app: &AppState,
    ws: &crate::workspace::Workspace,
    area: Rect,
    terminal_runtimes: &TerminalRuntimeRegistry,
) -> usize {
    let mut best_scroll = ws.active_tab;
    let mut best_distance = u16::MAX;
    let viewport_center = area.x.saturating_mul(2).saturating_add(area.width);

    for scroll in 0..=ws.active_tab {
        let rects = layout_tab_hit_areas(app, ws, area, scroll, terminal_runtimes);
        let Some(active_rect) = rects.get(ws.active_tab).copied() else {
            continue;
        };
        if active_rect.width == 0 {
            continue;
        }

        let active_center = active_rect
            .x
            .saturating_mul(2)
            .saturating_add(active_rect.width);
        let distance = active_center.abs_diff(viewport_center);
        if distance <= best_distance {
            best_distance = distance;
            best_scroll = scroll;
        }
    }

    best_scroll
}

fn trailing_tab_controls_x(tab_hit_areas: &[Rect], fallback_x: u16) -> u16 {
    tab_hit_areas
        .iter()
        .rev()
        .find(|rect| rect.width > 0)
        .map(|rect| rect.x + rect.width)
        .unwrap_or(fallback_x)
}

fn max_tab_scroll(
    app: &AppState,
    ws: &crate::workspace::Workspace,
    area: Rect,
    terminal_runtimes: &TerminalRuntimeRegistry,
) -> usize {
    (0..ws.tabs.len())
        .find(|&scroll| {
            layout_tab_hit_areas(app, ws, area, scroll, terminal_runtimes)
                .last()
                .is_some_and(|rect| rect.width > 0)
        })
        .unwrap_or(0)
}

pub(crate) fn compute_tab_bar_view(
    app: &AppState,
    ws: &crate::workspace::Workspace,
    area: Rect,
    current_scroll: usize,
    follow_active: bool,
    mouse_chrome: bool,
    terminal_runtimes: &TerminalRuntimeRegistry,
) -> TabBarView {
    let mut area = area;
    if app.tab_bar_vertical_padding && area.height > 1 {
        area.y = area.y.saturating_add(1);
        area.height = area.height.saturating_sub(1);
    }

    if area.width == 0 || area.height == 0 {
        return TabBarView::default();
    }

    if !mouse_chrome {
        let max_scroll = max_tab_scroll(app, ws, area, terminal_runtimes);
        let scroll = if follow_active {
            centered_tab_scroll(app, ws, area, terminal_runtimes).min(max_scroll)
        } else {
            current_scroll.min(max_scroll)
        };
        return TabBarView {
            scroll,
            tab_hit_areas: layout_tab_hit_areas(app, ws, area, scroll, terminal_runtimes),
            scroll_left_hit_area: Rect::default(),
            scroll_right_hit_area: Rect::default(),
            new_tab_hit_area: Rect::default(),
        };
    }

    let area_right = area.x + area.width;
    let all_tabs_area = Rect::new(
        area.x,
        area.y,
        area.width.saturating_sub(NEW_TAB_WIDTH),
        area.height,
    );
    let all_tabs = layout_tab_hit_areas(app, ws, all_tabs_area, 0, terminal_runtimes);
    let overflow = all_tabs.iter().any(|rect| rect.width == 0);
    if !overflow {
        let new_tab_x = trailing_tab_controls_x(&all_tabs, area.x);
        let new_tab_hit_area = Rect::new(
            new_tab_x,
            area.y,
            area_right.saturating_sub(new_tab_x).min(NEW_TAB_WIDTH),
            1,
        );
        return TabBarView {
            scroll: 0,
            tab_hit_areas: all_tabs,
            scroll_left_hit_area: Rect::default(),
            scroll_right_hit_area: Rect::default(),
            new_tab_hit_area,
        };
    }

    let left_hit_area = Rect::new(area.x, area.y, TAB_SCROLL_BUTTON_WIDTH.min(area.width), 1);
    let tab_area_x = left_hit_area.x + left_hit_area.width;
    let reserved_trailing_width = NEW_TAB_WIDTH.saturating_add(TAB_SCROLL_BUTTON_WIDTH);
    let tab_area_right = area_right.saturating_sub(reserved_trailing_width);
    let tab_area = Rect::new(
        tab_area_x,
        area.y,
        tab_area_right.saturating_sub(tab_area_x),
        area.height,
    );

    let max_scroll = max_tab_scroll(app, ws, tab_area, terminal_runtimes);
    let scroll = if follow_active {
        centered_tab_scroll(app, ws, tab_area, terminal_runtimes).min(max_scroll)
    } else {
        current_scroll.min(max_scroll)
    };
    let tab_hit_areas = layout_tab_hit_areas(app, ws, tab_area, scroll, terminal_runtimes);
    let trailing_x = trailing_tab_controls_x(&tab_hit_areas, tab_area_x).min(tab_area_right);
    let right_hit_area = Rect::new(
        trailing_x,
        area.y,
        area_right
            .saturating_sub(trailing_x)
            .min(TAB_SCROLL_BUTTON_WIDTH),
        1,
    );
    let new_tab_x = right_hit_area.x + right_hit_area.width;
    let new_tab_hit_area = Rect::new(
        new_tab_x,
        area.y,
        area_right.saturating_sub(new_tab_x).min(NEW_TAB_WIDTH),
        1,
    );

    TabBarView {
        scroll,
        tab_hit_areas,
        scroll_left_hit_area: left_hit_area,
        scroll_right_hit_area: right_hit_area,
        new_tab_hit_area,
    }
}

fn tab_drop_indicator_x(
    app: &AppState,
    ws: &crate::workspace::Workspace,
    insert_idx: usize,
) -> Option<u16> {
    let mut visible_tabs = app
        .view
        .tab_hit_areas
        .iter()
        .enumerate()
        .filter(|(_, rect)| rect.width > 0);
    let first_visible = visible_tabs.clone().next()?;
    let last_visible = visible_tabs.next_back().unwrap_or(first_visible);

    if insert_idx == 0 {
        return Some(if first_visible.0 == 0 {
            first_visible.1.x
        } else {
            app.view.tab_scroll_left_hit_area.x + app.view.tab_scroll_left_hit_area.width
        });
    }

    if let Some((_, rect)) = app
        .view
        .tab_hit_areas
        .iter()
        .enumerate()
        .find(|(idx, rect)| *idx == insert_idx && rect.width > 0)
    {
        return Some(rect.x.saturating_sub(1));
    }

    if insert_idx >= ws.tabs.len() {
        return Some(if last_visible.0 + 1 >= ws.tabs.len() {
            last_visible.1.x + last_visible.1.width
        } else {
            app.view.tab_scroll_right_hit_area.x.saturating_sub(1)
        });
    }

    None
}

pub(super) fn render_tab_bar(
    app: &AppState,
    terminal_runtimes: &TerminalRuntimeRegistry,
    frame: &mut Frame,
    area: Rect,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let Some(active_ws_idx) = app.active else {
        return;
    };
    let Some(ws) = app.workspaces.get(active_ws_idx) else {
        return;
    };

    let p = &app.palette;
    let content_area = {
        let mut c = area;
        if app.tab_bar_vertical_padding && c.height > 1 {
            c.y = c.y.saturating_add(1);
            c.height = c.height.saturating_sub(1);
        }
        if app.tab_bar_left_padding {
            Rect::new(
                c.x.saturating_add(1),
                c.y,
                c.width.saturating_sub(1),
                c.height,
            )
        } else {
            c
        }
    };

    frame.render_widget(
        Paragraph::new(" ".repeat(area.width as usize)).style(Style::default().bg(p.panel_bg)),
        area,
    );

    let first_visible_idx = app
        .view
        .tab_hit_areas
        .iter()
        .enumerate()
        .find(|(_, rect)| rect.width > 0)
        .map(|(idx, _)| idx);
    let last_visible_idx = app
        .view
        .tab_hit_areas
        .iter()
        .enumerate()
        .rev()
        .find(|(_, rect)| rect.width > 0)
        .map(|(idx, _)| idx);
    let can_scroll_left = app.view.tab_scroll_left_hit_area.width > 0 && app.tab_scroll > 0;
    let can_scroll_right = app.view.tab_scroll_right_hit_area.width > 0
        && last_visible_idx.is_some_and(|idx| idx + 1 < ws.tabs.len());

    if app.mouse_capture && app.view.tab_scroll_left_hit_area.width > 0 {
        let style = if can_scroll_left {
            Style::default().fg(p.overlay1).bg(p.surface0)
        } else {
            Style::default()
                .fg(p.overlay0)
                .bg(p.surface0)
                .add_modifier(Modifier::DIM)
        };
        frame.render_widget(
            Paragraph::new(" < ").style(style),
            app.view.tab_scroll_left_hit_area,
        );
    }

    if app.mouse_capture && app.view.tab_scroll_right_hit_area.width > 0 {
        let style = if can_scroll_right {
            Style::default().fg(p.overlay1).bg(p.surface0)
        } else {
            Style::default()
                .fg(p.overlay0)
                .bg(p.surface0)
                .add_modifier(Modifier::DIM)
        };
        frame.render_widget(
            Paragraph::new(" > ").style(style),
            app.view.tab_scroll_right_hit_area,
        );
    }

    for (idx, tab) in ws.tabs.iter().enumerate() {
        let Some(rect) = app.view.tab_hit_areas.get(idx).copied() else {
            break;
        };
        if rect.width == 0 {
            continue;
        }
        let active = idx == ws.active_tab;
        let dim_auto_name = app.dim_auto_named_tabs && tab.is_auto_named();

        let num_style;
        let name_style;
        if active {
            let mut ns = Style::default().fg(panel_contrast_fg(p)).bg(p.accent);
            let mut nms = Style::default().fg(p.text).bg(p.surface0);
            if dim_auto_name {
                ns = ns.add_modifier(Modifier::DIM);
                nms = nms.add_modifier(Modifier::DIM);
            } else {
                ns = ns.add_modifier(Modifier::BOLD);
                nms = nms.add_modifier(Modifier::BOLD);
            }
            num_style = ns;
            name_style = nms;
        } else {
            let mut ns = Style::default().fg(p.overlay0).bg(p.panel_bg);
            let mut nms = Style::default().fg(p.overlay1).bg(p.panel_bg);
            if dim_auto_name {
                ns = ns.add_modifier(Modifier::DIM);
                nms = nms.add_modifier(Modifier::DIM);
            }
            num_style = ns;
            name_style = nms;
        }

        let name = tab_title_with_zoom(app, ws, idx, terminal_runtimes);
        let num_str = format!(" {} ", idx + 1);
        let name_str = format!(" {} ", name);

        let rect_width = rect.width as usize;
        let num_width = num_str.chars().count();

        let (disp_num, disp_name) = if rect_width <= num_width {
            let truncated_num: String = num_str.chars().take(rect_width).collect();
            (truncated_num, String::new())
        } else {
            let name_avail = rect_width - num_width;
            if name_avail >= name_str.chars().count() {
                (num_str, name_str)
            } else if name_avail <= 3 {
                let truncated: String = name_str.chars().take(name_avail).collect();
                (num_str, truncated)
            } else {
                let max_chars = name_avail.saturating_sub(3);
                let name_part: String = name.chars().take(max_chars).collect();
                (num_str, format!(" {}… ", name_part))
            }
        };

        let spans = vec![
            Span::styled(disp_num, num_style),
            Span::styled(disp_name, name_style),
        ];
        frame.render_widget(Paragraph::new(Line::from(spans)), rect);
    }

    if let Some(crate::app::state::DragState {
        target:
            crate::app::state::DragTarget::TabReorder {
                ws_idx,
                insert_idx: Some(insert_idx),
                ..
            },
    }) = &app.drag
    {
        if *ws_idx == active_ws_idx {
            if let Some(x) = tab_drop_indicator_x(app, ws, *insert_idx) {
                frame.buffer_mut()[(
                    x.min(content_area.x + content_area.width.saturating_sub(1)),
                    content_area.y,
                )]
                    .set_symbol("│")
                    .set_style(Style::default().fg(p.accent));
            }
        }
    }

    if app.mouse_capture && app.view.new_tab_hit_area.width > 0 {
        frame.render_widget(
            Paragraph::new(" + ").style(Style::default().fg(p.overlay1)),
            app.view.new_tab_hit_area,
        );
    }

    if first_visible_idx.is_some_and(|idx| idx > 0) {
        let x = if app.mouse_capture && app.view.tab_scroll_left_hit_area.width > 0 {
            app.view.tab_scroll_left_hit_area.x + app.view.tab_scroll_left_hit_area.width
        } else {
            content_area.x
        };
        if x < content_area.x + content_area.width {
            frame.buffer_mut()[(x, content_area.y)]
                .set_symbol("…")
                .set_style(Style::default().fg(p.overlay0));
        }
    }
    if last_visible_idx.is_some_and(|idx| idx + 1 < ws.tabs.len()) {
        let x = if app.mouse_capture && app.view.tab_scroll_right_hit_area.width > 0 {
            app.view.tab_scroll_right_hit_area.x.saturating_sub(1)
        } else {
            content_area.x + content_area.width.saturating_sub(1)
        };
        if x >= content_area.x && x < content_area.x + content_area.width {
            frame.buffer_mut()[(x, content_area.y)]
                .set_symbol("…")
                .set_style(Style::default().fg(p.overlay0));
        }
    }

    let mut is_git = app.mode == Mode::DiffViewer;
    if !is_git {
        if let Some(tab) = ws.tabs.get(ws.active_tab) {
            let pane_id = tab.layout.focused();
            if let Some(rt) =
                app.runtime_for_pane_in_workspace(terminal_runtimes, active_ws_idx, pane_id)
            {
                if let Some(cwd) = rt.cwd() {
                    is_git = crate::workspace::git_repo_root(&cwd).is_some();
                }
            }
        }
    }

    let button_width = 9;
    if is_git && area.width > button_width + 10 {
        let button_x = area.x + area.width.saturating_sub(button_width);
        let button_rect = Rect::new(button_x, content_area.y, button_width, 1);

        let (icon_style, text_style) = if app.mode == Mode::DiffViewer {
            (
                Style::default().fg(p.panel_bg).bg(p.accent),
                Style::default().fg(p.text).bg(p.surface0),
            )
        } else {
            (
                Style::default().fg(p.text).bg(p.surface1),
                Style::default().fg(p.overlay1).bg(p.surface0),
            )
        };

        let spans = vec![
            Span::styled(" Δ ", icon_style),
            Span::styled(" diff ", text_style),
        ];
        frame.render_widget(Paragraph::new(Line::from(spans)), button_rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;
    use crate::workspace::Workspace;
    use ratatui::{backend::TestBackend, Terminal};

    fn buffer_row_text(buffer: &ratatui::buffer::Buffer, area: Rect, row: u16) -> String {
        (area.x..area.x + area.width)
            .map(|x| buffer[(x, row)].symbol())
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    #[test]
    fn tab_bar_marks_zoomed_tabs_without_renaming_them() {
        let mut app = AppState::test_new();
        let mut ws = Workspace::test_new("test");
        ws.tabs[0].zoomed = true;
        let custom_tab = ws.test_add_tab(Some("test"));
        ws.tabs[custom_tab].zoomed = true;

        app.workspaces = vec![ws];
        app.active = Some(0);
        app.view.tab_bar_rect = Rect::new(0, 0, 30, 1);
        let runtimes = TerminalRuntimeRegistry::new();
        let view = compute_tab_bar_view(
            &app,
            &app.workspaces[0],
            app.view.tab_bar_rect,
            0,
            true,
            false,
            &runtimes,
        );
        app.view.tab_hit_areas = view.tab_hit_areas;

        let backend = TestBackend::new(30, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_tab_bar(&app, &runtimes, frame, app.view.tab_bar_rect))
            .unwrap();

        let row = buffer_row_text(terminal.backend().buffer(), app.view.tab_bar_rect, 0);
        assert!(row.contains(" 1  zsh Z"), "tab row: {row:?}");
        assert!(row.contains(" 2  test Z"), "tab row: {row:?}");
        assert_eq!(app.workspaces[0].tab_display_name(0).as_deref(), Some("1"));
        assert_eq!(
            app.workspaces[0].tab_display_name(custom_tab).as_deref(),
            Some("test")
        );
    }

    #[test]
    fn zoom_marker_counts_toward_tab_width() {
        let app = AppState::test_new();
        let mut ws = Workspace::test_new("test");
        ws.tabs[0].set_custom_name("abcdefgh".into());
        ws.tabs[0].zoomed = true;

        let runtimes = TerminalRuntimeRegistry::new();
        assert_eq!(tab_width(&app, &ws, 0, &runtimes), 15);
    }
}
