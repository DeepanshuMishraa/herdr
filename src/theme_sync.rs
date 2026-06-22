use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Per-tool theme mapping for a single theme name.
struct ToolThemeMapping {
    pi_agent: &'static str,
    zed: &'static str,
    tmux_flavour: Option<&'static str>,
    ghostty_bg: &'static str,
    opencode: &'static str,
    hacktui: &'static str,
}

fn resolve_tool_mapping(herdr_name: &str) -> Option<&'static ToolThemeMapping> {
    let key = herdr_name.to_lowercase().replace([' ', '_'], "-");
    let mapping = match key.as_str() {
        "catppuccin" => &ToolThemeMapping {
            pi_agent: "catppuccin-macchiato",
            zed: "Catppuccin Mocha",
            tmux_flavour: Some("macchiato"),
            ghostty_bg: "#1e1e2e",
            opencode: "catppuccin-macchiato",
            hacktui: "catppuccin-macchiato",
        },
        "catppuccin-macchiato" | "macchiato" => &ToolThemeMapping {
            pi_agent: "catppuccin-macchiato",
            zed: "Catppuccin Mocha",
            tmux_flavour: Some("macchiato"),
            ghostty_bg: "#24273a",
            opencode: "catppuccin-macchiato",
hacktui: "catppuccin-macchiato",
        },
        "catppuccin-mocha" => &ToolThemeMapping {
            pi_agent: "catppuccin-macchiato",
            zed: "Catppuccin Mocha",
            tmux_flavour: Some("macchiato"),
            ghostty_bg: "#1e1e2e",
            opencode: "catppuccin-macchiato",
hacktui: "catppuccin-macchiato",
        },
        "catppuccin-latte" | "latte" | "light" => &ToolThemeMapping {
            pi_agent: "catppuccin-latte",
            zed: "Catppuccin Latte",
            tmux_flavour: Some("latte"),
            ghostty_bg: "#eff1f5",
            opencode: "catppuccin-latte",
hacktui: "catppuccin-frappe",
        },
        "tokyo-night" | "tokyonight" => &ToolThemeMapping {
            pi_agent: "tokyo-night",
            zed: "Tokyo Night",
            tmux_flavour: None,
            ghostty_bg: "#1a1b26",
            opencode: "tokyo-night",
hacktui: "tokyo-night",
        },
        "tokyo-night-day" | "tokyo-day" | "tokyonight-day" => &ToolThemeMapping {
            pi_agent: "tokyo-night",
            zed: "Tokyo Night Day",
            tmux_flavour: None,
            ghostty_bg: "#e1e2e7",
            opencode: "tokyo-night-day",
hacktui: "tokyo-night",
        },
        "dracula" => &ToolThemeMapping {
            pi_agent: "dracula",
            zed: "Dracula",
            tmux_flavour: None,
            ghostty_bg: "#282a36",
            opencode: "dracula",
hacktui: "dracula",
        },
        "nord" => &ToolThemeMapping {
            pi_agent: "nord",
            zed: "Nord",
            tmux_flavour: None,
            ghostty_bg: "#2e3440",
            opencode: "nord",
hacktui: "nord",
        },
        "gruvbox" | "gruvbox-dark" => &ToolThemeMapping {
            pi_agent: "gruvbox",
            zed: "Gruvbox",
            tmux_flavour: None,
            ghostty_bg: "#282828",
            opencode: "gruvbox",
hacktui: "gruvbox",
        },
        "gruvbox-light" => &ToolThemeMapping {
            pi_agent: "gruvbox-light",
            zed: "Gruvbox Light",
            tmux_flavour: None,
            ghostty_bg: "#fbf1c7",
            opencode: "gruvbox-light",
hacktui: "flexoki",
        },
        "one-dark" | "onedark" => &ToolThemeMapping {
            pi_agent: "one-dark",
            zed: "One Dark",
            tmux_flavour: None,
            ghostty_bg: "#282c34",
            opencode: "one-dark",
hacktui: "one-dark",
        },
        "one-light" | "onelight" => &ToolThemeMapping {
            pi_agent: "one-light",
            zed: "One Light",
            tmux_flavour: None,
            ghostty_bg: "#fafafa",
            opencode: "one-light",
hacktui: "github",
        },
        "solarized" | "solarized-dark" => &ToolThemeMapping {
            pi_agent: "solarized",
            zed: "Solarized Dark",
            tmux_flavour: None,
            ghostty_bg: "#002b36",
            opencode: "solarized",
hacktui: "solarized",
        },
        "solarized-light" => &ToolThemeMapping {
            pi_agent: "solarized-light",
            zed: "Solarized Light",
            tmux_flavour: None,
            ghostty_bg: "#fdf6e3",
            opencode: "solarized-light",
hacktui: "github",
        },
        "kanagawa" => &ToolThemeMapping {
            pi_agent: "kanagawa",
            zed: "Kanagawa",
            tmux_flavour: None,
            ghostty_bg: "#1f1f28",
            opencode: "kanagawa",
hacktui: "kanagawa",
        },
        "kanagawa-lotus" | "lotus" => &ToolThemeMapping {
            pi_agent: "kanagawa-lotus",
            zed: "Kanagawa Lotus",
            tmux_flavour: None,
            ghostty_bg: "#f4edd9",
            opencode: "kanagawa-lotus",
hacktui: "kanagawa",
        },
        "rose-pine" | "rosepine" => &ToolThemeMapping {
            pi_agent: "rose-pine",
            zed: "Rosé Pine",
            tmux_flavour: None,
            ghostty_bg: "#191724",
            opencode: "rose-pine",
hacktui: "rose-pine",
        },
        "rose-pine-dawn" | "rosepine-dawn" | "dawn" => &ToolThemeMapping {
            pi_agent: "rose-pine",
            zed: "Rosé Pine Dawn",
            tmux_flavour: None,
            ghostty_bg: "#faf4ed",
            opencode: "rose-pine-dawn",
hacktui: "rose-pine",
        },
        "vesper" => &ToolThemeMapping {
            pi_agent: "vesper",
            zed: "Vesper",
            tmux_flavour: None,
            ghostty_bg: "#101010",
            opencode: "vesper",
hacktui: "vesper",
        },
        _ => return None,
    };
    Some(mapping)
}

pub struct DetectedTool {
    pub name: &'static str,
    pub config_path: PathBuf,
}

pub struct ToolApplyResult {
    pub name: &'static str,
    pub config_path: PathBuf,
    pub status: ApplyStatus,
}

pub enum ApplyStatus {
    Updated,
    Skipped(String),
    Error(String),
}

/// Result of applying a theme across all tools.
pub(crate) struct ApplyResult {
    pub results: Vec<ToolApplyResult>,
}

/// Centralized global theme state stored at ~/.config/themes/settings.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalThemeState {
    pub theme: String,
    pub applied_at: String,
    pub tools: BTreeMap<String, ToolStateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStateEntry {
    pub theme: String,
    pub config: String,
}

pub fn color_to_hex(color: &Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Reset => "default".into(),
        Color::Black => "#000000".into(),
        Color::Red => "#ff0000".into(),
        Color::Green => "#00ff00".into(),
        Color::Yellow => "#ffff00".into(),
        Color::Blue => "#0000ff".into(),
        Color::Magenta => "#ff00ff".into(),
        Color::Cyan => "#00ffff".into(),
        Color::Gray => "#808080".into(),
        Color::DarkGray => "#404040".into(),
        Color::LightRed => "#ff8080".into(),
        Color::LightGreen => "#80ff80".into(),
        Color::LightYellow => "#ffff80".into(),
        Color::LightBlue => "#8080ff".into(),
        Color::LightMagenta => "#ff80ff".into(),
        Color::LightCyan => "#80ffff".into(),
        Color::White => "#ffffff".into(),
        Color::Indexed(i) => format!("#{i:02x}{i:02x}{i:02x}"),
    }
}

fn home_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            return PathBuf::from(profile);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else {
        PathBuf::from("/")
    }
}

/// Detect installed tools by checking their config paths.
pub fn detect_tools() -> Vec<DetectedTool> {
    let mut tools = Vec::new();
    let home = home_dir();

    tools.push(DetectedTool {
        name: "herdr",
        config_path: crate::config::config_path(),
    });

    let pi_path = home.join(".pi/agent/settings.json");
    if pi_path.exists() {
        tools.push(DetectedTool {
            name: "pi-agent",
            config_path: pi_path,
        });
    }

    let nvim_candidates = [
        home.join(".config/nvim/lua/plugins/color-scheme.lua"),
        home.join(".config/nvim/lua/dipxsy/plugins/color-scheme.lua"),
        home.join(".config/nvim/init.lua"),
    ];
    for p in &nvim_candidates {
        if p.exists() {
            tools.push(DetectedTool {
                name: "neovim",
                config_path: p.clone(),
            });
            break;
        }
    }

    let zed_path = home.join(".config/zed/settings.json");
    if zed_path.exists() {
        tools.push(DetectedTool {
            name: "zed",
            config_path: zed_path,
        });
    }

    let tmux_path = home.join(".config/tmux/tmux.conf");
    if tmux_path.exists() {
        tools.push(DetectedTool {
            name: "tmux",
            config_path: tmux_path,
        });
    }

    let ghostty_path = home.join(".config/ghostty/config");
    if ghostty_path.exists() {
        tools.push(DetectedTool {
            name: "ghostty",
            config_path: ghostty_path,
        });
    }

    let opencode_path = home.join(".config/opencode/tui.json");
    if opencode_path.exists() {
        tools.push(DetectedTool {
            name: "opencode",
            config_path: opencode_path,
        });
    }

    // Hacktui: check for binary and cache directory
    let hacktui_paths = [
        home.join(".bun/bin/hacktui"),
        PathBuf::from("/usr/local/bin/hacktui"),
    ];
    if hacktui_paths.iter().any(|p| p.exists()) {
        tools.push(DetectedTool {
            name: "hacktui",
            config_path: std::env::temp_dir().join("hacktui/query-cache.json"),
        });
    }

    tools
}

/// Global theme state file path: ~/.config/themes/settings.json
fn global_state_path() -> PathBuf {
    home_dir().join(".config/themes/settings.json")
}

/// Read the centralized global theme state file.
pub fn read_global_state() -> Option<GlobalThemeState> {
    let path = global_state_path();
    if !path.exists() {
        return None;
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<GlobalThemeState>(&content) {
            Ok(state) => Some(state),
            Err(err) => {
                warn!(
                    "failed to parse global theme state at {}: {err}",
                    path.display()
                );
                None
            }
        },
        Err(err) => {
            warn!(
                "failed to read global theme state at {}: {err}",
                path.display()
            );
            None
        }
    }
}

fn write_global_state(theme_name: &str, results: &[ToolApplyResult]) {
    let path = global_state_path();
    let mut tools = BTreeMap::new();
    for r in results {
        if matches!(r.status, ApplyStatus::Updated) {
            tools.insert(
                r.name.to_string(),
                ToolStateEntry {
                    theme: theme_name.to_string(),
                    config: r.config_path.display().to_string(),
                },
            );
        }
    }

    let state = GlobalThemeState {
        theme: theme_name.to_string(),
        applied_at: chrono_or_default(),
        tools,
    };

    if let Some(parent) = path.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            warn!("failed to create {}: {err}", parent.display());
            return;
        }
    }

    match serde_json::to_string_pretty(&state) {
        Ok(json) => {
            if let Err(err) = std::fs::write(&path, json) {
                warn!(
                    "failed to write global theme state to {}: {err}",
                    path.display()
                );
            }
        }
        Err(err) => {
            warn!("failed to serialize global theme state: {err}");
        }
    }
}

fn chrono_or_default() -> String {
    // Best-effort ISO 8601 timestamp; no chrono dependency.
    use std::time::{SystemTime, UNIX_EPOCH};
    let Ok(dur) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return "unknown".into();
    };
    let secs = dur.as_secs();
    // Simple UTC datetime from Unix timestamp (no chrono dep)
    let days_since_epoch = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Gregorian calendar date from days since epoch
    let (year, month, day) = days_to_date(days_since_epoch);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn days_to_date(days: u64) -> (u64, u64, u64) {
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y as u64) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let year = y as u64;
    let mut remaining = remaining as u64;
    let mdays = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    for (i, &days_in_month) in mdays.iter().enumerate() {
        if remaining < days_in_month {
            return (year, (i + 1) as u64, remaining + 1);
        }
        remaining -= days_in_month;
    }
    (year, 12, 31)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Write a theme name to a tool's config file.
/// Returns the tool-specific theme name that was applied, or an error.
fn apply_to_tool(
    tool: &DetectedTool,
    pal: &crate::app::state::Palette,
    mapping: &ToolThemeMapping,
) -> ToolApplyResult {
    let status = match tool.name {
        "herdr" => apply_to_herdr(tool, mapping),
        "pi-agent" => apply_to_pi_agent(tool, mapping, pal),
        "neovim" => apply_to_neovim(tool, mapping, pal),
        "zed" => apply_to_zed(tool, mapping, pal),
        "tmux" => apply_to_tmux(tool, mapping),
        "ghostty" => apply_to_ghostty(tool, mapping),
        "opencode" => apply_to_opencode(tool, mapping, pal),
        "hacktui" => apply_to_hacktui(tool, mapping, pal),
        other => ApplyStatus::Skipped(format!("unknown tool: {other}")),
    };

    ToolApplyResult {
        name: tool.name,
        config_path: tool.config_path.clone(),
        status,
    }
}

fn apply_to_herdr(tool: &DetectedTool, mapping: &ToolThemeMapping) -> ApplyStatus {
    let content = match read_config_text(&tool.config_path) {
        Ok(c) => c,
        Err(_) => String::new(),
    };

    let quoted = toml::Value::String(mapping.opencode.into()).to_string();
    let updated = crate::config::upsert_section_value(&content, "theme", "name", &quoted);

    let result = write_config(&tool.config_path, &updated);

    // Try to reload the running herdr server so the theme takes effect immediately
    let _ = request_herdr_reload();

    result
}

/// Attempt to tell the running herdr server to reload config. Best-effort.
fn request_herdr_reload() {
    use crate::api::client::ApiClient;
    use crate::api::schema::{EmptyParams, Method, Request};

    let client = ApiClient::local();
    let _ = client.request_value(&Request {
        id: "cli:theme:reload-config".into(),
        method: Method::ServerReloadConfig(EmptyParams::default()),
    });
}

fn apply_to_pi_agent(
    tool: &DetectedTool,
    mapping: &ToolThemeMapping,
    pal: &crate::app::state::Palette,
) -> ApplyStatus {
    // Update the theme name in settings.json
    let content = match read_config_text(&tool.config_path) {
        Ok(c) => c,
        Err(e) => return ApplyStatus::Error(e),
    };

    let mut value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(err) => {
            return ApplyStatus::Error(format!("invalid JSON: {err}"));
        }
    };

    value["theme"] = serde_json::Value::String(mapping.pi_agent.to_string());

    let updated = match serde_json::to_string_pretty(&value) {
        Ok(s) => s,
        Err(err) => return ApplyStatus::Error(format!("serialization failed: {err}")),
    };

    if let Err(err) = std::fs::write(&tool.config_path, updated + "\n") {
        return ApplyStatus::Error(format!("write failed: {err}"));
    }

    // Also generate a full Pi theme file at ~/.pi/agent/themes/<name>.json
    generate_pi_theme_file(tool.config_path.parent(), mapping, pal);

    ApplyStatus::Updated
}

/// Generate a full Pi agent theme JSON file from the herdr Palette.
fn generate_pi_theme_file(
    pi_config_dir: Option<&Path>,
    mapping: &ToolThemeMapping,
    pal: &crate::app::state::Palette,
) {
    let Some(config_dir) = pi_config_dir else {
        return;
    };

    // Pi config dir is ~/.pi/agent, themes dir is ~/.pi/agent/themes
    let themes_dir = config_dir.join("themes");
    let theme_path = themes_dir.join(format!("{}.json", mapping.pi_agent));

    // Don't overwrite an existing theme file (respects user customizations)
    if theme_path.exists() {
        return;
    }

    let vars = pi_vars_from_palette(pal);
    let export = pi_export_from_palette(pal);

    #[derive(Serialize)]
    struct PiThemeFile<'a> {
        name: &'a str,
        vars: BTreeMap<&'a str, String>,
        #[serde(serialize_with = "serialize_colors_ref")]
        colors: &'a [(&'a str, &'a str)],
        export: BTreeMap<&'a str, String>,
    }

    fn serialize_colors_ref<S>(colors: &&[(&str, &str)], s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(Some(colors.len()))?;
        for (k, v) in *colors {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }

    let colors: &[(&str, &str)] = &PI_COLORS;

    let theme = PiThemeFile {
        name: mapping.pi_agent,
        vars,
        colors,
        export,
    };

    if let Some(parent) = theme_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match serde_json::to_string_pretty(&theme) {
        Ok(json) => {
            if let Err(err) = std::fs::write(&theme_path, json) {
                warn!(
                    "failed to write Pi theme file to {}: {err}",
                    theme_path.display()
                );
            }
        }
        Err(err) => {
            warn!("failed to serialize Pi theme file: {err}");
        }
    }
}

const PI_COLORS: &[(&str, &str)] = &[
    ("accent", "blue"),
    ("border", "surface0"),
    ("borderAccent", "surface1"),
    ("borderMuted", "surface2"),
    ("success", "green"),
    ("error", "red"),
    ("warning", "yellow"),
    ("muted", "subtext1"),
    ("dim", "overlay1"),
    ("text", "text"),
    ("thinkingText", "overlay2"),
    ("selectedBg", "surface1"),
    ("userMessageBg", "mantle"),
    ("userMessageText", "text"),
    ("customMessageBg", "crust"),
    ("customMessageText", "text"),
    ("customMessageLabel", "pink"),
    ("toolPendingBg", "base"),
    ("toolSuccessBg", "mantle"),
    ("toolErrorBg", "toolErrorBg"),
    ("toolTitle", "blue"),
    ("toolOutput", "subtext1"),
    ("mdHeading", "mauve"),
    ("mdLink", "blue"),
    ("mdLinkUrl", "sky"),
    ("mdCode", "green"),
    ("mdCodeBlock", "text"),
    ("mdCodeBlockBorder", "surface2"),
    ("mdQuote", "yellow"),
    ("mdQuoteBorder", "yellow"),
    ("mdHr", "subtext0"),
    ("mdListBullet", "blue"),
    ("toolDiffAdded", "green"),
    ("toolDiffRemoved", "red"),
    ("toolDiffContext", "overlay2"),
    ("syntaxComment", "overlay2"),
    ("syntaxKeyword", "mauve"),
    ("syntaxFunction", "blue"),
    ("syntaxVariable", "red"),
    ("syntaxString", "green"),
    ("syntaxNumber", "peach"),
    ("syntaxType", "yellow"),
    ("syntaxOperator", "sky"),
    ("syntaxPunctuation", "text"),
    ("thinkingOff", "surface2"),
    ("thinkingMinimal", "surface1"),
    ("thinkingLow", "blue"),
    ("thinkingMedium", "teal"),
    ("thinkingHigh", "mauve"),
    ("thinkingXhigh", "pink"),
    ("bashMode", "peach"),
];

fn pi_vars_from_palette(pal: &crate::app::state::Palette) -> BTreeMap<&'static str, String> {
    let mut vars = BTreeMap::new();

    // Map herdr palette colors to Pi vars (best-effort approximation)
    let mut set = |name: &'static str, color: &Color| {
        vars.insert(name, color_to_hex(color));
    };

    // Direct mappings from herdr palette to Pi vars
    set("text", &pal.text);
    set("subtext0", &pal.subtext0);
    set("overlay0", &pal.overlay0);
    set("overlay1", &pal.overlay1);
    set("surface0", &pal.surface0);
    set("surface1", &pal.surface1);
    set("mauve", &pal.mauve);
    set("green", &pal.green);
    set("yellow", &pal.yellow);
    set("red", &pal.red);
    set("blue", &pal.blue);
    set("teal", &pal.teal);
    set("peach", &pal.peach);

    // Derived/approximate colors
    set("surface2", &pal.separator);
    set("base", &pal.panel_bg);
    set("mantle", &pal.active_space_bg);
    set("crust", &pal.surface_dim);

    // Subtext1: slightly brighter than subtext0
    set("subtext1", &pal.overlay1);

    // Overlay2: between overlay1 and surface0
    set("overlay2", &pal.overlay0);

    // Colors not directly in herdr palette — approximate from nearby
    set("rosewater", &pal.text);
    set("flamingo", &pal.peach);
    set("pink", &pal.mauve);
    set("maroon", &pal.red);
    set("sky", &pal.blue);
    set("sapphire", &pal.teal);
    set("lavender", &pal.accent);
    set("toolErrorBg", &pal.surface1);

    vars
}

fn pi_export_from_palette(pal: &crate::app::state::Palette) -> BTreeMap<&'static str, String> {
    let mut export = BTreeMap::new();
    export.insert("pageBg", color_to_hex(&pal.active_space_bg));
    export.insert("cardBg", color_to_hex(&pal.panel_bg));
    export.insert("infoBg", color_to_hex(&pal.surface0));
    export
}

fn apply_to_neovim(
    tool: &DetectedTool,
    mapping: &ToolThemeMapping,
    pal: &crate::app::state::Palette,
) -> ApplyStatus {
    let content = match read_config_text(&tool.config_path) {
        Ok(c) => c,
        Err(e) => return ApplyStatus::Error(e),
    };

    // Step 1: Generate a standalone colorscheme file at
    // ~/.config/nvim/lua/herdr-sync.lua that neovim can `require()`.
    // The parent of the plugin dir is the lua dir: lua/plugins/.. = lua/
    // This avoids placing code after `return { ... }` in the main config.
    let nvim_lua_dir = tool.config_path.parent().and_then(|p| p.parent()).unwrap_or(tool.config_path.parent().unwrap());
    generate_neovim_herdr_sync_file(nvim_lua_dir, pal, mapping);

    // Step 2: Replace vim.cmd.colorscheme("...") or require("herdr-sync") with
    // require("herdr-sync"), and strip any old herdr-sync block leftover code
    // (everything after a known marker line OR after the module return block).
    // The old herdr-sync block may be:
    //   1. Prefixed with `-- herdr-sync:`, or
    //   2. Code starting with `vim.g.colors_name = "herdr-sync"` (if marker was
    //      stripped by a previous version)
    let mut found_colorscheme = false;
    let mut stripping = false;
    let mut lines: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();

        // Detect old herdr-sync block: marker comment or the known code pattern
        if trimmed.starts_with("-- herdr-sync:")
            || trimmed.starts_with("vim.g.colors_name = \"herdr-sync\"")
            || trimmed == "local h = vim.api.nvim_set_hl"
        {
            stripping = true;
        }
        if stripping {
            continue;
        }

        if !found_colorscheme
            && (trimmed.starts_with("vim.cmd.colorscheme(")
                || trimmed.starts_with("colorscheme ")
                || trimmed == "require(\"herdr-sync\")")
        {
            let indent = &line[..line.len() - line.trim_start().len()];
            lines.push(format!("{indent}require(\"herdr-sync\")"));
            found_colorscheme = true;
            continue;
        }
        lines.push(line.to_string());
    }

    if !found_colorscheme {
        return ApplyStatus::Skipped("no colorscheme line found".into());
    }

    write_config(&tool.config_path, &lines.join("\n"))
}

fn generate_neovim_herdr_sync_file(config_dir: &Path, pal: &crate::app::state::Palette, _mapping: &ToolThemeMapping) {
    let file_path = config_dir.join("herdr-sync.lua");
    if let Some(parent) = file_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let bg = color_to_hex(&pal.panel_bg);
    let fg = color_to_hex(&pal.text);
    let surface0 = color_to_hex(&pal.surface0);
    let surface1 = color_to_hex(&pal.surface1);
    let surface_dim = color_to_hex(&pal.surface_dim);
    let separator = color_to_hex(&pal.separator);
    let overlay0 = color_to_hex(&pal.overlay0);
    let overlay1 = color_to_hex(&pal.overlay1);
    let accent = color_to_hex(&pal.accent);
    let mauve = color_to_hex(&pal.mauve);
    let green = color_to_hex(&pal.green);
    let yellow = color_to_hex(&pal.yellow);
    let red = color_to_hex(&pal.red);
    let blue = color_to_hex(&pal.blue);
    let teal = color_to_hex(&pal.teal);
    let peach = color_to_hex(&pal.peach);
    let active_bg = color_to_hex(&pal.active_space_bg);

    // ── normal ─────────────────────────────────────────────────────────────────
    // fg, bg, cursorline, cursorlinenr, linenr, visual, search, incsearch
    // statusline, separator, tabs, pmenu, float, signcolumn, folded, cursor
    // matchparen, colorcolumn, conceal, directory, specialkey, nontext, endofbuffer
    // title, whitespace
    // ── syntax ────────────────────────────────────────────────────────────────
    // comment, constant, string, character, number, boolean, float
    // identifier, function, statement, conditional, repeat, label
    // operator, keyword, exception, preproc, include, define, macro
    // type, storageclass, structure, typedef, special, underlined, error, todo
    // ── diagnostics ───────────────────────────────────────────────────────────
    // diagnosticeerror, diagnosticwarn, diagnosticinfo, diagnostichint
    // ── diff ──────────────────────────────────────────────────────────────────
    // diffadd, diffchange, diffdelete, difftext
    // ── spell & lsp ───────────────────────────────────────────────────────────
    // spellbad, spellcap, spelllocal, spellrare, lspreferencetext, lspreferenceread, lspreferencewrite
    let lua = format!(
        r##"-- Generated by `herdr theme apply` — do not edit manually
vim.g.colors_name = "herdr-sync"

local h = vim.api.nvim_set_hl

-- Normal / background
h(0, "Normal", {{ fg = "{fg}", bg = "{bg}" }})
h(0, "NormalFloat", {{ fg = "{fg}", bg = "{bg}" }})
h(0, "CursorLine", {{ bg = "{surface0}" }})
h(0, "CursorLineNr", {{ fg = "{overlay1}" }})
h(0, "LineNr", {{ fg = "{overlay0}" }})
h(0, "Visual", {{ bg = "{surface1}" }})
h(0, "VisualNOS", {{ bg = "{surface1}" }})
h(0, "Search", {{ bg = "{surface1}", fg = "{fg}" }})
h(0, "IncSearch", {{ bg = "{accent}", fg = "{surface_dim}" }})
h(0, "CurSearch", {{ bg = "{accent}", fg = "{surface_dim}" }})
h(0, "StatusLine", {{ fg = "{fg}", bg = "{active_bg}" }})
h(0, "StatusLineNC", {{ fg = "{overlay0}", bg = "{surface_dim}" }})
h(0, "WinSeparator", {{ fg = "{separator}" }})
h(0, "TabLine", {{ fg = "{overlay0}", bg = "{surface_dim}" }})
h(0, "TabLineSel", {{ fg = "{fg}", bg = "{surface0}" }})
h(0, "TabLineFill", {{ bg = "{surface_dim}" }})
h(0, "Pmenu", {{ fg = "{fg}", bg = "{surface0}" }})
h(0, "PmenuSel", {{ bg = "{surface1}", fg = "{fg}" }})
h(0, "PmenuSbar", {{ bg = "{surface_dim}" }})
h(0, "PmenuThumb", {{ bg = "{separator}" }})
h(0, "FloatBorder", {{ fg = "{separator}" }})
h(0, "FloatTitle", {{ fg = "{accent}" }})
h(0, "SignColumn", {{ bg = "{bg}" }})
h(0, "Folded", {{ fg = "{overlay0}", bg = "{surface_dim}" }})
h(0, "FoldColumn", {{ fg = "{overlay0}", bg = "{bg}" }})
h(0, "Cursor", {{ fg = "{bg}", bg = "{fg}" }})
h(0, "lCursor", {{ fg = "{bg}", bg = "{fg}" }})
h(0, "MatchParen", {{ bg = "{surface1}", fg = "{accent}" }})
h(0, "ColorColumn", {{ bg = "{surface0}" }})
h(0, "Conceal", {{ fg = "{overlay0}" }})
h(0, "Directory", {{ fg = "{blue}" }})
h(0, "SpecialKey", {{ fg = "{peach}" }})
h(0, "NonText", {{ fg = "{overlay0}" }})
h(0, "EndOfBuffer", {{ fg = "{overlay0}" }})
h(0, "Title", {{ fg = "{accent}", bold = true }})
h(0, "Whitespace", {{ fg = "{overlay0}" }})

-- Syntax groups
h(0, "Comment", {{ fg = "{overlay0}", italic = true }})
h(0, "Constant", {{ fg = "{peach}" }})
h(0, "String", {{ fg = "{green}" }})
h(0, "Character", {{ fg = "{green}" }})
h(0, "Number", {{ fg = "{peach}" }})
h(0, "Boolean", {{ fg = "{peach}" }})
h(0, "Float", {{ fg = "{peach}" }})
h(0, "Identifier", {{ fg = "{blue}" }})
h(0, "Function", {{ fg = "{blue}" }})
h(0, "Statement", {{ fg = "{mauve}" }})
h(0, "Conditional", {{ fg = "{mauve}" }})
h(0, "Repeat", {{ fg = "{mauve}" }})
h(0, "Label", {{ fg = "{mauve}" }})
h(0, "Operator", {{ fg = "{teal}" }})
h(0, "Keyword", {{ fg = "{mauve}" }})
h(0, "Exception", {{ fg = "{mauve}" }})
h(0, "PreProc", {{ fg = "{yellow}" }})
h(0, "Include", {{ fg = "{blue}" }})
h(0, "Define", {{ fg = "{yellow}" }})
h(0, "Macro", {{ fg = "{yellow}" }})
h(0, "Type", {{ fg = "{teal}" }})
h(0, "StorageClass", {{ fg = "{yellow}" }})
h(0, "Structure", {{ fg = "{teal}" }})
h(0, "Typedef", {{ fg = "{teal}" }})
h(0, "Special", {{ fg = "{mauve}" }})
h(0, "Underlined", {{ fg = "{blue}", underline = true }})
h(0, "Error", {{ fg = "{red}", bg = "{bg}" }})
h(0, "Todo", {{ fg = "{yellow}", bg = "{surface_dim}" }})

-- Diagnostics
h(0, "DiagnosticError", {{ fg = "{red}" }})
h(0, "DiagnosticWarn", {{ fg = "{yellow}" }})
h(0, "DiagnosticInfo", {{ fg = "{blue}" }})
h(0, "DiagnosticHint", {{ fg = "{teal}" }})
h(0, "DiagnosticUnderlineError", {{ undercurl = true, sp = "{red}" }})
h(0, "DiagnosticUnderlineWarn", {{ undercurl = true, sp = "{yellow}" }})
h(0, "DiagnosticUnderlineInfo", {{ undercurl = true, sp = "{blue}" }})
h(0, "DiagnosticUnderlineHint", {{ undercurl = true, sp = "{teal}" }})

-- Diff
h(0, "DiffAdd", {{ fg = "{green}", bg = "{surface_dim}" }})
h(0, "DiffChange", {{ fg = "{yellow}", bg = "{surface_dim}" }})
h(0, "DiffDelete", {{ fg = "{red}", bg = "{surface_dim}" }})
h(0, "DiffText", {{ fg = "{accent}", bg = "{surface_dim}" }})

-- Spell
h(0, "SpellBad", {{ undercurl = true, sp = "{red}" }})
h(0, "SpellCap", {{ undercurl = true, sp = "{yellow}" }})
h(0, "SpellLocal", {{ undercurl = true, sp = "{blue}" }})
h(0, "SpellRare", {{ undercurl = true, sp = "{mauve}" }})

-- LSP references
h(0, "LspReferenceText", {{ bg = "{surface1}" }})
h(0, "LspReferenceRead", {{ bg = "{surface1}" }})
h(0, "LspReferenceWrite", {{ bg = "{surface1}" }})
"##,
    );

    let _ = std::fs::write(&file_path, lua);
}

fn apply_to_zed(tool: &DetectedTool, mapping: &ToolThemeMapping, pal: &crate::app::state::Palette) -> ApplyStatus {
    let content = match read_config_text(&tool.config_path) {
        Ok(c) => c,
        Err(e) => return ApplyStatus::Error(e),
    };

    // Step 1: Generate a zed theme file at ~/.config/zed/themes/herdr.json
    // with colors from herdr's palette. Zed discovers themes from this directory.
    let themes_dir = tool.config_path.parent().unwrap().join("themes");
    let _ = std::fs::create_dir_all(&themes_dir);
    generate_zed_theme_file(&themes_dir, "herdr", pal);

    // Step 2: Update settings.json to point to "herdr dark"
    let cleaned = strip_jsonc_comments(&content);
    let last_obj = find_last_json_object(&cleaned);

    let last_valid = match last_obj {
        Some(obj) => obj,
        None => return apply_to_zed_line_based(tool, mapping, &content),
    };

    let mut value: serde_json::Value = match serde_json::from_str(last_valid) {
        Ok(v) => v,
        Err(_) => return apply_to_zed_line_based(tool, mapping, &content),
    };

    // Set the theme to reference our generated "herdr dark" theme
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "theme".into(),
            serde_json::json!({
                "mode": "system",
                "light": "herdr light",
                "dark": "herdr dark",
            }),
        );
    }

    let updated = match serde_json::to_string_pretty(&value) {
        Ok(s) => s,
        Err(err) => return ApplyStatus::Error(format!("serialization failed: {err}")),
    };

    write_config(&tool.config_path, &format!("{updated}\n"))
}

/// Generate a zed theme family file at ~/.config/zed/themes/herdr.json
/// with dark and light variants mapped from herdr's palette.
fn generate_zed_theme_file(themes_dir: &Path, name: &str, pal: &crate::app::state::Palette) {
    let h = |c: &Color| color_to_hex(c);
    let bg = h(&pal.panel_bg);
    let fg = h(&pal.text);
    let surface0 = h(&pal.surface0);
    let surface1 = h(&pal.surface1);
    let surface_dim = h(&pal.surface_dim);
    let sep = h(&pal.separator);
    let sub = h(&pal.subtext0);
    let over0 = h(&pal.overlay0);
    let over1 = h(&pal.overlay1);
    let acc = h(&pal.accent);
    let mauve = h(&pal.mauve);
    let green = h(&pal.green);
    let yellow = h(&pal.yellow);
    let red = h(&pal.red);
    let blue = h(&pal.blue);
    let teal = h(&pal.teal);
    let peach = h(&pal.peach);
    let active_bg = h(&pal.active_space_bg);

    let mut style: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    macro_rules! kv {
        ($k:expr, $v:expr) => { style.insert($k.into(), serde_json::Value::String(::std::clone::Clone::clone(&$v))); };
    }
    macro_rules! kvn {
        ($k:expr) => { style.insert($k.into(), serde_json::Value::Null); };
    }
    kv!("background", bg);
    kv!("foreground", fg);
    kv!("text", fg);
    kv!("text.muted", sub);
    kv!("text.accent", acc);
    kv!("text.disabled", over0);
    kv!("text.placeholder", over0);
    kv!("border", sep);
    kv!("border.variant", surface0);
    kv!("border.selected", acc);
    kv!("border.focused", acc);
    kv!("icon", sub);
    kv!("icon.accent", acc);
    kv!("icon.muted", over0);
    kv!("icon.disabled", over1);
    kv!("surface.background", surface0);
    kv!("elevated_surface.background", surface1);
    kv!("element.background", surface1);
    kv!("element.hover", surface1);
    kv!("element.active", surface0);
    kv!("element.selected", surface0);
    kv!("element.disabled", surface_dim);
    kv!("ghost_element.background", bg);
    kv!("ghost_element.hover", surface0);
    kv!("ghost_element.selected", surface0);
    kv!("ghost_element.active", surface_dim);
    kv!("ghost_element.disabled", surface_dim);
    kv!("panel.background", active_bg);
    kv!("panel.focused_border", acc);
    kv!("tab_bar.background", surface_dim);
    kv!("tab.active_background", surface0);
    kv!("tab.inactive_background", surface_dim);
    kv!("title_bar.background", surface_dim);
    kv!("title_bar.inactive_background", surface_dim);
    kv!("status_bar.background", active_bg);
    kv!("toolbar.background", bg);
    kv!("editor.background", bg);
    kv!("editor.foreground", fg);
    kv!("editor.active_line.background", surface0);
    kv!("editor.highlighted_line.background", surface0);
    kv!("editor.line_number", over0);
    kv!("editor.active_line_number", over1);
    kv!("editor.gutter.background", bg);
    kv!("editor.indent_guide", surface0);
    kv!("editor.indent_guide_active", sep);
    kv!("editor.invisible", over0);
    kv!("editor.wrap_guide", surface0);
    kv!("editor.active_wrap_guide", sep);
    kv!("editor.document_highlight.read_background", surface1);
    kv!("editor.document_highlight.write_background", surface1);
    kv!("editor.document_highlight.bracket_background", surface1);
    kv!("pane.focused_border", acc);
    kv!("pane_group.border", sep);
    kv!("scrollbar.track.background", bg);
    kv!("scrollbar.track.border", sep);
    kv!("scrollbar.thumb.background", surface0);
    kv!("scrollbar.thumb.border", sep);
    kv!("scrollbar.thumb.hover_background", surface1);
    kv!("search.match_background", surface1);
    kv!("drop_target.background", surface1);

    kv!("error", red);
    kvn!("error.background");
    kv!("error.border", red);
    kv!("warning", yellow);
    kvn!("warning.background");
    kv!("warning.border", yellow);
    kv!("info", blue);
    kvn!("info.background");
    kv!("info.border", blue);
    kv!("hint", teal);
    kvn!("hint.background");
    kv!("hint.border", teal);
    kv!("success", green);
    kvn!("success.background");
    kv!("success.border", green);
    kv!("conflict", peach);
    kvn!("conflict.background");
    kv!("conflict.border", peach);
    kv!("created", green);
    kvn!("created.background");
    kv!("created.border", green);
    kv!("deleted", red);
    kvn!("deleted.background");
    kv!("deleted.border", red);
    kv!("modified", yellow);
    kvn!("modified.background");
    kv!("modified.border", yellow);
    kv!("renamed", blue);
    kvn!("renamed.background");
    kv!("renamed.border", blue);
    kv!("ignored", over0);
    kvn!("ignored.background");
    kv!("ignored.border", over0);
    kv!("hidden", over0);
    kvn!("hidden.background");
    kv!("hidden.border", over0);
    kv!("unreachable", over0);
    kvn!("unreachable.background");
    kv!("unreachable.border", over0);
    kv!("predictive", over0);
    kvn!("predictive.background");
    kv!("predictive.border", over0);
    kv!("link_text.hover", acc);
    kv!("terminal.background", bg);
    kv!("terminal.foreground", fg);
    kv!("terminal.bright_foreground", fg);
    kv!("terminal.dim_foreground", over0);
    kv!("terminal.ansi.black", surface_dim);
    kv!("terminal.ansi.bright_black", over0);
    kv!("terminal.ansi.dim_black", surface_dim);
    kv!("terminal.ansi.red", red);
    kv!("terminal.ansi.bright_red", red);
    kv!("terminal.ansi.dim_red", red);
    kv!("terminal.ansi.green", green);
    kv!("terminal.ansi.bright_green", green);
    kv!("terminal.ansi.dim_green", green);
    kv!("terminal.ansi.yellow", yellow);
    kv!("terminal.ansi.bright_yellow", yellow);
    kv!("terminal.ansi.dim_yellow", yellow);
    kv!("terminal.ansi.blue", blue);
    kv!("terminal.ansi.bright_blue", blue);
    kv!("terminal.ansi.dim_blue", blue);
    kv!("terminal.ansi.magenta", mauve);
    kv!("terminal.ansi.bright_magenta", mauve);
    kv!("terminal.ansi.dim_magenta", mauve);
    kv!("terminal.ansi.cyan", teal);
    kv!("terminal.ansi.bright_cyan", teal);
    kv!("terminal.ansi.dim_cyan", teal);
    kv!("terminal.ansi.white", fg);
    kv!("terminal.ansi.bright_white", fg);
    kv!("terminal.ansi.dim_white", sub);
    style.insert("accents".into(), serde_json::Value::Array(vec![serde_json::Value::String(acc)]));

    // Syntax
    let mut syntax = serde_json::Map::new();
    syntax.insert("comment".into(), serde_json::json!({ "color": over0.clone(), "font_style": "italic" }));
    let sc = |c: String| serde_json::json!({ "color": c });
    syntax.insert("constant".into(), sc(peach.clone()));
    syntax.insert("string".into(), sc(green.clone()));
    syntax.insert("character".into(), sc(green.clone()));
    syntax.insert("number".into(), sc(peach.clone()));
    syntax.insert("boolean".into(), sc(peach.clone()));
    syntax.insert("float".into(), sc(peach.clone()));
    syntax.insert("builtin".into(), serde_json::json!({ "color": red.clone() }));
    syntax.insert("function".into(), sc(blue.clone()));
    syntax.insert("method".into(), sc(blue.clone()));
    syntax.insert("constructor".into(), sc(blue.clone()));
    syntax.insert("variable".into(), sc(fg.clone()));
    syntax.insert("type".into(), sc(teal.clone()));
    syntax.insert("keyword".into(), serde_json::json!({ "color": mauve.clone(), "font_style": "italic" }));
    syntax.insert("label".into(), sc(mauve.clone()));
    syntax.insert("namespace".into(), sc(teal.clone()));
    syntax.insert("tag".into(), serde_json::json!({ "color": red.clone() }));
    syntax.insert("parameter".into(), sc(fg.clone()));
    syntax.insert("property".into(), sc(fg.clone()));
    syntax.insert("field".into(), sc(fg.clone()));
    syntax.insert("attribute".into(), sc(yellow.clone()));
    syntax.insert("operator".into(), sc(teal.clone()));
    syntax.insert("punctuation".into(), sc(fg.clone()));
    syntax.insert("punctuation.bracket".into(), sc(fg.clone()));
    syntax.insert("punctuation.delimiter".into(), sc(teal.clone()));
    syntax.insert("punctuation.special".into(), sc(teal.clone()));
    syntax.insert("string.special".into(), sc(green.clone()));
    syntax.insert("string.regex".into(), sc(peach.clone()));
    syntax.insert("string.escape".into(), sc(mauve.clone()));
    syntax.insert("embedded".into(), sc(fg.clone()));
    syntax.insert("comment.doc".into(), serde_json::json!({ "color": over0, "font_style": "italic" }));

    style.insert("syntax".into(), serde_json::Value::Object(syntax));

    let dark_theme = serde_json::json!({
        "name": format!("{name} dark"),
        "appearance": "dark",
        "style": style,
    });

    let light_theme = serde_json::json!({
        "name": format!("{name} light"),
        "appearance": "light",
        "style": style,
    });

    let theme_family = serde_json::json!({
        "name": name,
        "author": "herdr",
        "themes": [dark_theme, light_theme],
    });

    let theme_path = themes_dir.join(format!("{name}.json"));
    match serde_json::to_string_pretty(&theme_family) {
        Ok(json) => {
            if let Err(err) = std::fs::write(&theme_path, json) {
                warn!("failed to write zed theme file: {err}");
            }
        }
        Err(err) => {
            warn!("failed to serialize zed theme: {err}");
        }
    }
}

/// Strip JSONC line comments (// ...) and trailing content from JSON.
fn strip_jsonc_comments(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Find the last complete JSON object `{...}` in the text, returning its content.
fn find_last_json_object(text: &str) -> Option<&str> {
    // Search from the end for a balanced `{` ... `}`
    let bytes = text.as_bytes();

    // Find last `}`
    let close = text.rfind('}')?;

    // Walk backwards from close to find matching `{`
    let mut depth = 0;
    let mut start = close;
    for pos in (0..=close).rev() {
        match bytes[pos] {
            b'}' => depth += 1,
            b'{' => {
                depth -= 1;
                if depth == 0 {
                    start = pos;
                    break;
                }
            }
            _ => {}
        }
    }

    if depth != 0 {
        return None;
    }

    Some(&text[start..=close])
}

/// Fallback: extract only the last complete JSON document, parse it with
/// trailing-comma support, update the theme, and write back a single clean file.
fn apply_to_zed_line_based(tool: &DetectedTool, mapping: &ToolThemeMapping, content: &str) -> ApplyStatus {
    // Strip JSONC line comments, remove trailing commas, then parse as strict JSON.
    // This handles Zed's JSONC format (comments + trailing commas).
    let cleaned = strip_jsonc_comments(content);
    let fixed = fix_trailing_commas(&cleaned);

    // Find the last complete JSON `{...}` in the fixed text
    let last_block = match find_last_json_object(&fixed) {
        Some(obj) => obj,
        None => return ApplyStatus::Error("no valid JSON object found".into()),
    };

    let mut value: serde_json::Value = match serde_json::from_str(last_block) {
        Ok(v) => v,
        Err(err) => {
            return ApplyStatus::Error(format!("failed to parse zed JSON: {err}"));
        }
    };

    // Update the theme field
    if let Some(obj) = value.as_object_mut() {
        if let Some(theme) = obj.get_mut("theme") {
            match theme {
                serde_json::Value::String(_) => {
                    *theme = serde_json::Value::String(mapping.zed.to_string());
                }
                serde_json::Value::Object(theme_obj) => {
                    if theme_obj.contains_key("dark") || theme_obj.contains_key("light") {
                        theme_obj.insert(
                            "dark".into(),
                            serde_json::Value::String(mapping.zed.to_string()),
                        );
                    } else {
                        *theme = serde_json::Value::String(mapping.zed.to_string());
                    }
                }
                _ => {
                    *theme = serde_json::Value::String(mapping.zed.to_string());
                }
            }
        } else {
            obj.insert(
                "theme".into(),
                serde_json::Value::String(mapping.zed.to_string()),
            );
        }
    }

    let updated = match serde_json::to_string_pretty(&value) {
        Ok(s) => s,
        Err(err) => return ApplyStatus::Error(format!("serialization failed: {err}")),
    };

    write_config(&tool.config_path, &format!("{updated}\n"))
}

/// Remove trailing commas before `}` or `]` in JSON text.
fn fix_trailing_commas(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b',' {
            // Look ahead past whitespace for `}` or `]`
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n' || bytes[j] == b'\r') {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                // Skip the comma — emit nothing
                i += 1;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn apply_to_tmux(tool: &DetectedTool, mapping: &ToolThemeMapping) -> ApplyStatus {
    let Some(flavour) = mapping.tmux_flavour else {
        return ApplyStatus::Skipped("theme is not catppuccin-based; tmux catppuccin plugin only supports catppuccin flavours".into());
    };

    let content = match read_config_text(&tool.config_path) {
        Ok(c) => c,
        Err(e) => return ApplyStatus::Error(e),
    };

    // Line-based replacement for @catppuccin_flavour '...'
    let mut found = false;
    let mut lines: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("set") && trimmed.contains("@catppuccin_flavour") {
            let indent = &line[..line.len() - line.trim_start().len()];
            // Replace single-quoted or double-quoted flavour value
            if let Some(start) = trimmed.find(|c| c == '\'' || c == '"') {
                let quote = trimmed.as_bytes()[start] as char;
                if let Some(end) = trimmed[start + 1..].find(quote) {
                    let before = &trimmed[..start + 1];
                    let after = &trimmed[start + 1 + end..];
                    lines.push(format!("{indent}{before}{flavour}{after}"));
                    found = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if !found {
        return ApplyStatus::Skipped("no @catppuccin_flavour setting found in tmux.conf".into());
    }

    write_config(&tool.config_path, &lines.join("\n"))
}

fn apply_to_ghostty(tool: &DetectedTool, mapping: &ToolThemeMapping) -> ApplyStatus {
    let content = match read_config_text(&tool.config_path) {
        Ok(c) => c,
        Err(e) => return ApplyStatus::Error(e),
    };

    let bg_hex = mapping.ghostty_bg;

    // Update or add background = <hex>
    let mut found_bg = false;
    let mut lines: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("background ") || trimmed.starts_with("background=") {
            let indent = &line[..line.len() - line.trim_start().len()];
            lines.push(format!("{indent}background = {bg_hex}"));
            found_bg = true;
        } else {
            lines.push(line.to_string());
        }
    }

    if !found_bg {
        lines.push(format!("background = {bg_hex}"));
    }

    write_config(&tool.config_path, &lines.join("\n"))
}

fn apply_to_hacktui(
    tool: &DetectedTool,
    mapping: &ToolThemeMapping,
    _pal: &crate::app::state::Palette,
) -> ApplyStatus {
    // Hacktui stores the active theme name in a JSON cache file at
    // <tmpdir>/hacktui/query-cache.json under the key "hacktui-theme".
    // The cache dir is created on first run. We write just the theme key.
    let cache_dir = tool.config_path.parent().unwrap();
    let _ = std::fs::create_dir_all(cache_dir);

    let mut cache = match read_config_text(&tool.config_path) {
        Ok(c) => serde_json::from_str::<serde_json::Value>(&c).unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        Err(_) => serde_json::Value::Object(serde_json::Map::new()),
    };

    if let Some(obj) = cache.as_object_mut() {
        obj.insert(
            "hacktui-theme".into(),
            serde_json::Value::String(mapping.hacktui.to_string()),
        );
    }

    let updated = match serde_json::to_string_pretty(&cache) {
        Ok(s) => s,
        Err(err) => return ApplyStatus::Error(format!("serialization failed: {err}")),
    };

    write_config(&tool.config_path, &updated)
}

fn apply_to_opencode(
    tool: &DetectedTool,
    _mapping: &ToolThemeMapping,
    pal: &crate::app::state::Palette,
) -> ApplyStatus {
    let content = match read_config_text(&tool.config_path) {
        Ok(c) => c,
        Err(e) => return ApplyStatus::Error(e),
    };

    // Step 1: Generate the "herdr" theme file at ~/.config/opencode/themes/herdr.json.
    // OpenCode discovers themes from the themes/ directory and loads the file by name.
    // We always write to "herdr" and regenerate it with the current palette on every
    // apply, so OpenCode's "herdr" theme always matches herdr's current colors.
    // Stale theme files from previous applies are cleaned up so only "herdr" remains.
    let themes_dir = tool.config_path.parent().unwrap().join("themes");
    clean_opencode_themes(&themes_dir);
    generate_opencode_theme_file(&themes_dir, "herdr", pal);

    // Step 2: Write the theme name ("herdr") to tui.json
    let mut lines: Vec<String> = Vec::new();
    let mut found_opening = false;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();

        if trimmed.starts_with("\"theme\":") || trimmed.starts_with("\"theme\" :") {
            continue;
        }

        if !lines.is_empty()
            && trimmed.starts_with('"')
            && lines.last().map_or(false, |l| {
                let t = l.trim();
                t.ends_with(']') || t.ends_with('}') || t.ends_with('"')
            })
        {
            let prev_trimmed = lines.last().unwrap().trim();
            if prev_trimmed != "{" && !prev_trimmed.ends_with(',') {
                if let Some(last) = lines.last_mut() {
                    last.push(',');
                }
            }
        }

        if !found_opening && trimmed == "{" {
            lines.push(raw_line.to_string());
            lines.push("  \"theme\": \"herdr\",".into());
            found_opening = true;
        } else {
            lines.push(raw_line.to_string());
        }
    }

    let result = if found_opening {
        write_config(&tool.config_path, &lines.join("\n"))
    } else {
        let fallback = serde_json::json!({
            "$schema": "https://opencode.ai/tui.json",
            "theme": "herdr",
        });
        match serde_json::to_string_pretty(&fallback) {
            Ok(s) => write_config(&tool.config_path, &s),
            Err(err) => ApplyStatus::Error(format!("serialization failed: {err}")),
        }
    };

    // Step 3: Try to send SIGUSR2 to tell a running OpenCode to refresh themes
    let _ = signal_opencode_refresh();

    result
}

/// Send SIGUSR2 to any running OpenCode process so it refreshes themes.
fn signal_opencode_refresh() -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::process::Command;
        let _ = Command::new("pkill")
            .args(["-SIGUSR2", "opencode"])
            .output();
    }
    Ok(())
}

/// Remove stale OpenCode theme files — only keep "herdr.json".
fn clean_opencode_themes(themes_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(themes_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name != "herdr.json" && name.to_string_lossy().ends_with(".json") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Generate a full OpenCode theme JSON file from herdr's palette.
fn generate_opencode_theme_file(themes_dir: &Path, name: &str, pal: &crate::app::state::Palette) {
    let theme_path = themes_dir.join(format!("{name}.json"));
    if let Some(parent) = theme_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let theme = opencode_theme_props(pal);
    #[derive(Serialize)]
    struct OpenCodeTheme {
        theme: BTreeMap<String, String>,
    }

    match serde_json::to_string_pretty(&OpenCodeTheme { theme }) {
        Ok(json) => {
            if let Err(err) = std::fs::write(&theme_path, json) {
                warn!("failed to write opencode theme: {err}");
            }
        }
        Err(err) => {
            warn!("failed to serialize opencode theme: {err}");
        }
    }
}

/// Map herdr palette colors to OpenCode ThemeJson color properties.
fn opencode_theme_props(pal: &crate::app::state::Palette) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    let h = |c: &Color| color_to_hex(c);

    let bg = h(&pal.panel_bg);
    let fg = h(&pal.text);
    let surface0 = h(&pal.surface0);
    let surface_dim = h(&pal.surface_dim);
    let sep = h(&pal.separator);
    let over0 = h(&pal.overlay0);
    let acc = h(&pal.accent);
    let mauve = h(&pal.mauve);
    let green = h(&pal.green);
    let yellow = h(&pal.yellow);
    let red = h(&pal.red);
    let blue = h(&pal.blue);
    let teal = h(&pal.teal);
    let peach = h(&pal.peach);

    m.insert("primary".into(), acc.clone());
    m.insert("secondary".into(), mauve.clone());
    m.insert("accent".into(), acc.clone());
    m.insert("error".into(), red.clone());
    m.insert("warning".into(), yellow.clone());
    m.insert("success".into(), green.clone());
    m.insert("info".into(), teal.clone());
    m.insert("text".into(), fg.clone());
    m.insert("textMuted".into(), over0.clone());
    m.insert("selectedListItemText".into(), bg.clone());
    m.insert("background".into(), bg.clone());
    m.insert("backgroundPanel".into(), surface_dim.clone());
    m.insert("backgroundElement".into(), surface0.clone());
    m.insert("backgroundMenu".into(), surface0.clone());
    m.insert("border".into(), sep.clone());
    m.insert("borderActive".into(), acc.clone());
    m.insert("borderSubtle".into(), surface0.clone());
    // Diff
    m.insert("diffAdded".into(), green.clone());
    m.insert("diffRemoved".into(), red.clone());
    m.insert("diffContext".into(), over0.clone());
    m.insert("diffHunkHeader".into(), over0.clone());
    m.insert("diffHighlightAdded".into(), green.clone());
    m.insert("diffHighlightRemoved".into(), red.clone());
    m.insert("diffAddedBg".into(), surface_dim.clone());
    m.insert("diffRemovedBg".into(), surface_dim.clone());
    m.insert("diffContextBg".into(), surface_dim.clone());
    m.insert("diffLineNumber".into(), over0.clone());
    m.insert("diffAddedLineNumberBg".into(), surface0.clone());
    m.insert("diffRemovedLineNumberBg".into(), surface0.clone());
    // Markdown
    m.insert("markdownText".into(), fg.clone());
    m.insert("markdownHeading".into(), acc.clone());
    m.insert("markdownLink".into(), blue.clone());
    m.insert("markdownLinkText".into(), teal.clone());
    m.insert("markdownCode".into(), green.clone());
    m.insert("markdownBlockQuote".into(), yellow.clone());
    m.insert("markdownEmph".into(), peach.clone());
    m.insert("markdownStrong".into(), fg.clone());
    m.insert("markdownHorizontalRule".into(), sep.clone());
    m.insert("markdownListItem".into(), blue.clone());
    m.insert("markdownListEnumeration".into(), acc.clone());
    m.insert("markdownImage".into(), blue.clone());
    m.insert("markdownImageText".into(), teal.clone());
    m.insert("markdownCodeBlock".into(), fg.clone());
    // Syntax
    m.insert("syntaxComment".into(), over0.clone());
    m.insert("syntaxKeyword".into(), mauve.clone());
    m.insert("syntaxFunction".into(), blue.clone());
    m.insert("syntaxVariable".into(), fg.clone());
    m.insert("syntaxString".into(), green.clone());
    m.insert("syntaxNumber".into(), peach.clone());
    m.insert("syntaxType".into(), teal.clone());
    m.insert("syntaxOperator".into(), teal);
    m.insert("syntaxPunctuation".into(), fg);
    m.insert("thinkingOpacity".into(), "0.6".into());

    m
}

fn read_config_text(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err("config file does not exist".into());
    }
    std::fs::read_to_string(path).map_err(|e| format!("read failed: {e}"))
}

fn write_config(path: &Path, content: &str) -> ApplyStatus {
    if let Some(parent) = path.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            return ApplyStatus::Error(format!("create dir failed: {err}"));
        }
    }
    match std::fs::write(path, content) {
        Ok(_) => ApplyStatus::Updated,
        Err(err) => ApplyStatus::Error(format!("write failed: {err}")),
    }
}

/// Validate that a theme name exists in herdr's built-in or external themes.
pub fn validate_theme_name(name: &str) -> bool {
    crate::app::state::Palette::from_name(name).is_some()
        || matches!(crate::config::load_external_theme(name), Ok(Some(_)))
}

/// List available built-in theme names.
pub fn list_builtin_theme_names() -> Vec<&'static str> {
    crate::app::state::THEME_NAMES.to_vec()
}

/// Apply a theme across all detected tools.
/// Returns a result with per-tool outcomes and optionally writes the global state file.
pub fn apply_theme(theme_name: &str, dry_run: bool) -> ApplyResult {
    let mapping = resolve_tool_mapping(theme_name);
    let pal = crate::app::state::Palette::from_name(theme_name);

    let tools = detect_tools();
    let mut results = Vec::new();

    for tool in &tools {
        let mapping = match mapping {
            Some(m) => m,
            None => {
                results.push(ToolApplyResult {
                    name: tool.name,
                    config_path: tool.config_path.clone(),
                    status: ApplyStatus::Skipped("no theme mapping available".into()),
                });
                continue;
            }
        };

        if dry_run {
            results.push(ToolApplyResult {
                name: tool.name,
                config_path: tool.config_path.clone(),
                status: ApplyStatus::Skipped("dry run".into()),
            });
            continue;
        }

        let pal = match &pal {
            Some(p) => p,
            None => {
                results.push(ToolApplyResult {
                    name: tool.name,
                    config_path: tool.config_path.clone(),
                    status: ApplyStatus::Skipped("theme not found in herdr".into()),
                });
                continue;
            }
        };

        let result = apply_to_tool(tool, pal, mapping);
        results.push(result);
    }

    if !dry_run {
        write_global_state(theme_name, &results);
    }

    ApplyResult { results }
}
