use crate::{
    export::timestamp,
    model::{sanitize_text, Model},
    Entry, Level,
};
use charm_core::{display_width, truncate};
use lipgloss::{Color, Style};

fn style(color: &str, bold: bool) -> Style {
    Style::new().foreground(Color::from(color)).bold(bold)
}

pub(crate) fn render(model: &Model) -> String {
    if model.width == 0 || model.height == 0 {
        return String::new();
    }
    let mut lines = vec![line(
        &format!(
            "{} {} · {} · {} · filtro: {}{}",
            model.len(),
            if model.len() == 1 {
                "entrada"
            } else {
                "entradas"
            },
            model.min_level,
            if model.follow {
                "follow on"
            } else {
                "follow off"
            },
            if model.filter.is_empty() {
                "-"
            } else {
                &model.filter
            },
            if model.searching { "▌" } else { "" }
        ),
        model.width,
        "#9CA3AF",
        true,
    )];
    if model.height >= 2 {
        lines.push(style("#3F3F46", false).render(&"─".repeat(model.width)));
    }
    let area = model.height.saturating_sub(lines.len());
    if area > 0 {
        let rows = model.visible();
        if rows.is_empty() {
            lines.push(line("(sem entradas)", model.width, "#6B7280", false));
        } else {
            let selected = model.selected(&rows);
            let end = (selected + 1).max(area).min(rows.len());
            let start = end.saturating_sub(area);
            for (idx, row) in rows.iter().enumerate().take(end).skip(start) {
                lines.push(entry_line(&row.entry, idx == selected, model.width));
            }
        }
    }
    lines.join("\n")
}

fn line(text: &str, width: usize, color: &str, bold: bool) -> String {
    style(color, bold).render(&truncate(text, width))
}

fn entry_line(entry: &Entry, selected: bool, width: usize) -> String {
    let mark = if selected { "› " } else { "  " };
    let (time, _) = timestamp(entry.time);
    let clock = time.split('T').nth(1).unwrap_or("00:00:00");
    let level_color = match entry.level {
        Level::Debug => "#6B7280",
        Level::Info => "#61AFEF",
        Level::Warn => "#E5C07B",
        Level::Error => "#E06C75",
    };
    let prefix = format!("{mark}{clock} {} ", entry.level.short());
    let mut rest = width.saturating_sub(display_width(&prefix));
    let source = if entry.source.is_empty() {
        String::new()
    } else {
        format!("[{}] ", sanitize_text(&entry.source))
    };
    let source = if rest >= 14 {
        truncate(&source, rest.saturating_sub(10))
    } else {
        String::new()
    };
    rest = rest.saturating_sub(display_width(&source));
    let message = truncate(&sanitize_text(&entry.message), rest);
    let marker = if selected {
        style("#F59E0B", true).render(mark)
    } else {
        mark.to_owned()
    };
    let prefix = format!(
        "{}{}",
        marker,
        style("#6B7280", false).render(&prefix[mark.len()..])
    );
    let colored_level = style(level_color, false).render(entry.level.short());
    let prefix = prefix.replace(entry.level.short(), &colored_level);
    let body = format!(
        "{}{}{}",
        prefix,
        style("#A78BFA", false).render(&source),
        message
    );
    Style::truncate_visible_line(&body, width)
}
