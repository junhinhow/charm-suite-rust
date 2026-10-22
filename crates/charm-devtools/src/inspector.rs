//! Text-mode DevTools panel renderer.

use std::fmt::Write as _;

use charmed_bubbletea::KeyType;

use crate::DevState;

/// Key used by the middleware to toggle the inspector.
pub const INSPECTOR_TOGGLE_KEY: KeyType = KeyType::F12;
const VISIBLE_LINES: usize = 20;

#[derive(Debug, Default)]
pub(crate) struct InspectorModel {
    scroll: usize,
    tab: usize,
}

impl InspectorModel {
    pub(crate) fn update(&mut self, key: KeyType) {
        match key {
            KeyType::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyType::Down => self.scroll = self.scroll.saturating_add(1),
            KeyType::Tab => {
                self.tab = (self.tab + 1) % 3;
                self.scroll = 0;
            }
            _ => {}
        }
    }

    pub(crate) fn render(&self, state: &DevState) -> String {
        let mut output = String::from(
            "╔══════════════════════════════════════════════════════╗\n\
             ║            Bubble Tea DevTools (F12 close)          ║\n\
             ╠══════════════════════════════════════════════════════╣\n",
        );
        let tabs = ["Messages", "State", "Components"];
        output.push_str("║ ");
        for (index, tab) in tabs.iter().enumerate() {
            if index == self.tab {
                let _ = write!(output, "[{tab}]");
            } else {
                let _ = write!(output, " {tab} ");
            }
            if index < tabs.len() - 1 {
                output.push_str(" | ");
            }
        }
        output.push_str("\n╠══════════════════════════════════════════════════════╣\n");

        match self.tab {
            0 => self.render_messages(state, &mut output),
            1 => self.render_state(state, &mut output),
            _ => Self::render_components(state, &mut output),
        }
        output.push_str(
            "╠══════════════════════════════════════════════════════╣\n\
             ║  ↑↓ scroll  |  Tab switch panel  |  F12 close      ║\n\
             ╚══════════════════════════════════════════════════════╝\n",
        );
        output
    }

    fn render_messages(&self, state: &DevState, output: &mut String) {
        let messages = state.messages();
        if messages.is_empty() {
            output.push_str("║  (no messages recorded yet)\n");
            return;
        }
        let start = messages.len().saturating_sub(VISIBLE_LINES + self.scroll);
        let end = (start + VISIBLE_LINES).min(messages.len());
        let _ = writeln!(
            output,
            "║  Message Log ({} total, showing {}-{})",
            messages.len(),
            start + 1,
            end
        );
        output.push_str("║  ────────────────────────────────────────────────\n");
        for entry in &messages[start..end] {
            let type_name: String = entry.type_name.chars().take(30).collect();
            let _ = writeln!(output, "║  {:?}  {type_name:<30}", entry.timestamp);
        }
    }

    fn render_state(&self, state: &DevState, output: &mut String) {
        output.push_str(
            "║  Current Model State\n║  ────────────────────────────────────────────────\n",
        );
        let lines: Vec<&str> = state.current_state().lines().collect();
        let start = self.scroll.min(lines.len().saturating_sub(1));
        for line in lines.iter().skip(start).take(20) {
            let truncated: String = line.chars().take(52).collect();
            let _ = writeln!(output, "║  {truncated}");
        }
    }

    fn render_components(state: &DevState, output: &mut String) {
        output.push_str("║  Component Tree\n║  ────────────────────────────────────────────────\n");
        for node in state.components() {
            render_node(node, 0, output);
        }
    }
}

fn render_node(node: &crate::ComponentNode, depth: usize, output: &mut String) {
    let indent = "  ".repeat(depth);
    let prefix = if depth == 0 { "●" } else { "├─" };
    let _ = writeln!(
        output,
        "║  {indent}{prefix} {} ({})",
        node.name, node.type_name
    );
    for child in &node.children {
        render_node(child, depth + 1, output);
    }
}
