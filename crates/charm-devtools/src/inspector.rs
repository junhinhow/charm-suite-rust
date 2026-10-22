//! Text-mode DevTools panel renderer.

use charm_core::{display_width, truncate};
use charmed_bubbletea::KeyType;

use crate::{ComponentNode, DevState};

/// Key used by the middleware to toggle the inspector.
pub const INSPECTOR_TOGGLE_KEY: KeyType = KeyType::F12;
const PANEL_INNER_WIDTH: usize = 54;
const CONTENT_WIDTH: usize = 52;
const VISIBLE_LINES: usize = 20;
const SEPARATOR: &str = "────────────────────────────────────────────────────";

#[derive(Debug, Default)]
pub(crate) struct InspectorModel {
    scroll: usize,
    tab: usize,
}

impl InspectorModel {
    pub(crate) fn update(&mut self, key: KeyType, state: &DevState) {
        match key {
            KeyType::Up => self.scroll = self.scroll.saturating_add(1).min(self.max_scroll(state)),
            KeyType::Down => self.scroll = self.scroll.saturating_sub(1),
            KeyType::Tab => {
                self.tab = (self.tab + 1) % 3;
                self.scroll = 0;
            }
            _ => {}
        }
    }

    pub(crate) fn render(&self, state: &DevState) -> String {
        let mut output = String::new();
        output.push('╔');
        output.push_str(&"═".repeat(PANEL_INNER_WIDTH));
        output.push_str("╗\n");
        write_box_centered(&mut output, "Bubble Tea DevTools (F12 close)");
        output.push('╠');
        output.push_str(&"═".repeat(PANEL_INNER_WIDTH));
        output.push_str("╣\n");
        write_box_line(&mut output, &self.tabs_line());
        output.push('╠');
        output.push_str(&"═".repeat(PANEL_INNER_WIDTH));
        output.push_str("╣\n");

        match self.tab {
            0 => self.render_messages(state, &mut output),
            1 => self.render_state(state, &mut output),
            _ => self.render_components(state, &mut output),
        }

        output.push('╠');
        output.push_str(&"═".repeat(PANEL_INNER_WIDTH));
        output.push_str("╣\n");
        write_box_line(&mut output, "↑↓ scroll  |  Tab switch panel  |  F12 close");
        output.push('╚');
        output.push_str(&"═".repeat(PANEL_INNER_WIDTH));
        output.push_str("╝\n");
        output
    }

    fn tabs_line(&self) -> String {
        ["Messages", "State", "Components"]
            .iter()
            .enumerate()
            .map(|(index, name)| {
                if index == self.tab {
                    format!("[{name}]")
                } else {
                    format!(" {name} ")
                }
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn max_scroll(&self, state: &DevState) -> usize {
        self.content_lines(state)
            .len()
            .saturating_sub(VISIBLE_LINES)
    }

    fn content_lines(&self, state: &DevState) -> Vec<String> {
        match self.tab {
            0 => state
                .messages()
                .iter()
                .map(|entry| {
                    format!(
                        "{}  {}",
                        format_timestamp(entry.timestamp),
                        truncate(&entry.type_name, 30)
                    )
                })
                .collect(),
            1 => wrap_text(state.current_state()),
            _ => state
                .components()
                .iter()
                .flat_map(|node| component_lines(node, 0))
                .collect(),
        }
    }

    fn render_messages(&self, state: &DevState, output: &mut String) {
        let messages = state.messages();
        if messages.is_empty() {
            write_box_line(output, "Message Log (0 total)");
            write_box_line(output, SEPARATOR);
            write_box_line(output, "(no messages recorded yet)");
            return;
        }
        let lines = self.content_lines(state);
        let (start, end) = visible_window(lines.len(), self.scroll);
        write_box_line(
            output,
            &format!(
                "Message Log ({} total, showing {}-{})",
                messages.len(),
                start + 1,
                end
            ),
        );
        write_box_line(output, SEPARATOR);
        for line in &lines[start..end] {
            write_box_line(output, line);
        }
    }

    fn render_state(&self, state: &DevState, output: &mut String) {
        write_box_line(output, "Current Model State");
        write_box_line(output, SEPARATOR);
        let lines = self.content_lines(state);
        if lines.is_empty() {
            write_box_line(output, "(no state captured yet)");
            return;
        }
        let (start, end) = visible_window(lines.len(), self.scroll);
        for line in &lines[start..end] {
            write_box_line(output, line);
        }
    }

    fn render_components(&self, state: &DevState, output: &mut String) {
        write_box_line(output, "Component Tree");
        write_box_line(output, SEPARATOR);
        let lines = self.content_lines(state);
        if lines.is_empty() {
            write_box_line(output, "(no components detected)");
            return;
        }
        let (start, end) = visible_window(lines.len(), self.scroll);
        for line in &lines[start..end] {
            write_box_line(output, line);
        }
    }
}

fn component_lines(node: &ComponentNode, depth: usize) -> Vec<String> {
    let indent = "  ".repeat(depth);
    let prefix = if depth == 0 { "●" } else { "├─" };
    let mut label = format!("{indent}{prefix} {} ({})", node.name, node.type_name);
    if node.cycle {
        label.push_str(" [cycle]");
    }
    let mut lines = vec![truncate(&label, CONTENT_WIDTH)];
    for child in &node.children {
        lines.extend(component_lines(child, depth + 1));
    }
    lines
}

fn wrap_text(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut chunk = String::new();
        let mut width = 0;
        for character in line.chars() {
            let char_width = display_width(&character.to_string());
            if width + char_width > CONTENT_WIDTH && !chunk.is_empty() {
                lines.push(std::mem::take(&mut chunk));
                width = 0;
            }
            chunk.push(character);
            width += char_width;
        }
        lines.push(chunk);
    }
    lines
}

fn visible_window(total: usize, scroll: usize) -> (usize, usize) {
    let scroll = scroll.min(total.saturating_sub(VISIBLE_LINES));
    let end = total.saturating_sub(scroll);
    (end.saturating_sub(VISIBLE_LINES), end)
}

fn format_timestamp(timestamp: std::time::SystemTime) -> String {
    let elapsed = timestamp
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = elapsed.as_secs() % 86_400;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        seconds / 3_600,
        (seconds / 60) % 60,
        seconds % 60,
        elapsed.subsec_millis()
    )
}

fn write_box_line(output: &mut String, text: &str) {
    output.push('║');
    let content = format!("  {}", truncate(text, PANEL_INNER_WIDTH.saturating_sub(2)));
    output.push_str(&pad_right(&content, PANEL_INNER_WIDTH));
    output.push_str("║\n");
}

fn write_box_centered(output: &mut String, text: &str) {
    output.push('║');
    let content = format!("  {text}");
    let width = display_width(&content);
    if width >= PANEL_INNER_WIDTH {
        output.push_str(&truncate(&content, PANEL_INNER_WIDTH));
    } else {
        let left = (PANEL_INNER_WIDTH - width) / 2;
        let right = PANEL_INNER_WIDTH - width - left;
        output.push_str(&" ".repeat(left));
        output.push_str(&content);
        output.push_str(&" ".repeat(right));
    }
    output.push_str("║\n");
}

fn pad_right(text: &str, width: usize) -> String {
    let visible = display_width(text);
    if visible >= width {
        truncate(text, width)
    } else {
        format!("{text}{}", " ".repeat(width - visible))
    }
}

#[cfg(test)]
mod tests {
    use super::{InspectorModel, PANEL_INNER_WIDTH};
    use crate::{ComponentNode, DevState, MessageEntry};
    use charm_core::display_width;
    use charmed_bubbletea::KeyType;
    use std::time::{Duration, SystemTime};

    fn state_with_messages(count: usize) -> DevState {
        let mut state = DevState::default();
        for index in 0..count {
            state.push_entry(MessageEntry {
                timestamp: SystemTime::UNIX_EPOCH
                    + Duration::from_secs(u64::try_from(index).unwrap_or(u64::MAX)),
                type_name: format!("message::{index}"),
                value: index.to_string(),
            });
        }
        state
    }

    #[test]
    fn scroll_is_bounded_and_up_reveals_older_messages() {
        let state = state_with_messages(30);
        let mut panel = InspectorModel::default();
        assert!(panel.render(&state).contains("showing 11-30"));
        panel.update(KeyType::Up, &state);
        assert!(panel.render(&state).contains("showing 10-29"));
        for _ in 0..50 {
            panel.update(KeyType::Up, &state);
        }
        assert!(panel.render(&state).contains("showing 1-20"));
        panel.update(KeyType::Down, &state);
        assert!(panel.render(&state).contains("showing 2-21"));
    }

    #[test]
    fn each_inspector_row_has_both_borders_and_fixed_display_width() {
        let state = state_with_messages(3);
        let mut panel = InspectorModel::default();
        for _ in 0..3 {
            let rendered = panel.render(&state);
            for line in rendered.trim_end().lines() {
                assert!(line
                    .chars()
                    .next()
                    .is_some_and(|character| "║╔╠╚".contains(character)));
                assert!(line
                    .chars()
                    .last()
                    .is_some_and(|character| "║╗╣╝".contains(character)));
                assert_eq!(display_width(line), PANEL_INNER_WIDTH + 2, "{line:?}");
            }
            panel.update(KeyType::Tab, &state);
        }
    }

    #[test]
    fn unicode_components_are_truncated_without_exceeding_panel_width() {
        let mut state = DevState::default();
        state.components.push(ComponentNode::new(
            "日本語の長いコンポーネント名を表示します",
            "module::Component",
            Vec::new(),
        ));
        let panel = InspectorModel {
            tab: 2,
            ..InspectorModel::default()
        };
        let rendered = panel.render(&state);
        for line in rendered.trim_end().lines() {
            assert!(display_width(line) <= PANEL_INNER_WIDTH + 2);
        }
    }
}
