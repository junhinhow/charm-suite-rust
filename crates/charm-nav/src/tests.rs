//! Unit tests ported from the Go `nav_test.go`, plus coverage of `add` and
//! `remove` on every component.

use lipgloss::utils::{strip_ansi, width_visible};

use crate::{Breadcrumb, Sidebar, Tabs};

/// Display column where `text` starts in `line`, ignoring ANSI codes.
fn column_of(line: &str, text: &str) -> usize {
    let plain = strip_ansi(line);
    let index = plain
        .find(text)
        .unwrap_or_else(|| panic!("texto {text:?} nao encontrado em {plain:?}"));
    width_visible(&plain[..index])
}

#[test]
fn sidebar_aligns_columns_with_a_multibyte_cursor() {
    // The default cursor "▸ " is 4 bytes and 2 columns. Measuring bytes would
    // shift the selected item two columns to the left.
    let mut sidebar = Sidebar::new(&["Dashboard", "Usuarios", "Configuracoes"]);
    sidebar.set_selected(1);

    let rendered = sidebar.render();
    let lines: Vec<&str> = rendered.lines().collect();
    assert_eq!(lines.len(), 3);

    let columns: Vec<usize> = vec![
        column_of(lines[0], "Dashboard"),
        column_of(lines[1], "Usuarios"),
        column_of(lines[2], "Configuracoes"),
    ];
    assert!(
        columns.iter().all(|c| *c == columns[0]),
        "colunas: {columns:?}"
    );
    assert_eq!(columns[0], 3, "1 de padding + 2 do cursor");
}

#[test]
fn sidebar_aligns_columns_with_an_ascii_cursor() {
    let mut sidebar = Sidebar::new(&["Dashboard", "Usuarios"]);
    sidebar.cursor = "> ".to_string();
    sidebar.set_selected(0);

    let rendered = sidebar.render();
    let lines: Vec<&str> = rendered.lines().collect();
    assert_eq!(
        column_of(lines[0], "Dashboard"),
        column_of(lines[1], "Usuarios")
    );
}

#[test]
fn sidebar_respects_a_three_column_cursor() {
    let mut sidebar = Sidebar::new(&["Dashboard", "Usuarios"]);
    sidebar.cursor = "=> ".to_string();
    sidebar.set_selected(1);

    let rendered = sidebar.render();
    let lines: Vec<&str> = rendered.lines().collect();
    let col0 = column_of(lines[0], "Dashboard");
    assert_eq!(col0, column_of(lines[1], "Usuarios"));
    assert_eq!(col0, 4);
}

#[test]
fn sidebar_truncates_long_items_instead_of_wrapping() {
    let long = "Este item tem um rotulo comprido demais para o menu lateral";
    let sidebar = Sidebar::new(&[long, "Usuarios"]);

    let rendered = sidebar.render();
    let lines: Vec<&str> = rendered.lines().collect();
    assert_eq!(lines.len(), 2, "sem quebra de linha: {rendered:?}");
    for line in &lines {
        assert!(width_visible(line) <= sidebar.width, "linha: {line:?}");
    }

    let plain = strip_ansi(lines[0]);
    assert!(!plain.contains(long), "esperava o item truncado: {plain:?}");
    assert!(plain.contains("Este item"), "inicio do rotulo: {plain:?}");
}

#[test]
fn sidebar_truncates_long_items_with_a_multibyte_cursor() {
    let mut sidebar = Sidebar::new(&["Item com acentuacao e emoji: configuracoes avancadas"]);
    sidebar.cursor = "▸ ".to_string();

    let rendered = sidebar.render();
    assert_eq!(rendered.lines().count(), 1);
    assert!(width_visible(&rendered) <= sidebar.width);
}

#[test]
fn sidebar_highlight_covers_the_whole_line() {
    let mut sidebar = Sidebar::new(&["Dashboard", "Usuarios"]);
    sidebar.set_selected(0);

    let rendered = sidebar.render();
    let selected = rendered.lines().next().unwrap_or("");
    assert_eq!(width_visible(selected), sidebar.width);
}

#[test]
fn sidebar_lines_all_have_the_same_width() {
    let mut sidebar = Sidebar::new(&["A", "Item bem mais longo que o primeiro", "C"]);
    sidebar.set_selected(1);

    for (index, line) in sidebar.render().lines().enumerate() {
        assert_eq!(width_visible(line), sidebar.width, "linha {index}");
    }
}

#[test]
fn sidebar_minimum_widths_are_respected() {
    for width in [0, 1, 2, 3, 5, 10] {
        let mut sidebar = Sidebar::new(&["Dashboard", "Usuarios"]);
        sidebar.width = width;

        let expected = width.max(1);
        for (index, line) in sidebar.render().lines().enumerate() {
            assert_eq!(width_visible(line), expected, "width={width} linha {index}");
        }
    }
}

#[test]
fn sidebar_with_an_empty_cursor_stays_aligned() {
    let mut sidebar = Sidebar::new(&["Dashboard", "Usuarios"]);
    sidebar.cursor = String::new();

    let rendered = sidebar.render();
    let lines: Vec<&str> = rendered.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(
        column_of(lines[0], "Dashboard"),
        column_of(lines[1], "Usuarios")
    );
}

#[test]
fn sidebar_cursor_wider_than_the_area_is_truncated() {
    let mut sidebar = Sidebar::new(&["Aa", "Bb"]);
    sidebar.cursor = ">>>>>>>>>>".to_string();

    let rendered = sidebar.render();
    let lines: Vec<&str> = rendered.lines().collect();
    for line in &lines {
        assert!(width_visible(line) <= sidebar.width, "linha: {line:?}");
    }
    assert_eq!(column_of(lines[0], "Aa"), column_of(lines[1], "Bb"));
}

#[test]
fn sidebar_add_appends_and_returns_the_first_index() {
    let mut sidebar = Sidebar::new(&["a", "b"]);
    assert_eq!(sidebar.add(&["c"]), Some(2));
    assert_eq!(sidebar.items(), ["a", "b", "c"]);
    assert_eq!(sidebar.add(&[]), None);
}

#[test]
fn sidebar_remove_returns_the_label_or_empty_when_invalid() {
    let mut sidebar = Sidebar::new(&["a", "b", "c"]);
    assert_eq!(sidebar.remove(1), "b");
    assert_eq!(sidebar.items(), ["a", "c"]);
    assert_eq!(sidebar.remove(99), "");
    assert_eq!(sidebar.items(), ["a", "c"]);
}

#[test]
fn sidebar_remove_before_the_selection_keeps_the_same_item() {
    let mut sidebar = Sidebar::new(&["a", "b", "c"]);
    sidebar.set_selected(2);
    sidebar.remove(0);
    assert_eq!(sidebar.selected(), 1);
    assert_eq!(sidebar.items()[sidebar.selected()], "c");
}

#[test]
fn sidebar_remove_of_the_selected_item_moves_to_the_next() {
    let mut sidebar = Sidebar::new(&["a", "b", "c"]);
    sidebar.set_selected(1);
    sidebar.remove(1);
    assert_eq!(sidebar.selected(), 1);
    assert_eq!(sidebar.items()[sidebar.selected()], "c");
}

#[test]
fn sidebar_remove_of_the_last_selected_item_moves_back() {
    let mut sidebar = Sidebar::new(&["a", "b", "c"]);
    sidebar.set_selected(2);
    sidebar.remove(2);
    assert_eq!(sidebar.selected(), 1);
    assert_eq!(sidebar.items()[sidebar.selected()], "b");
}

#[test]
fn sidebar_set_selected_ignores_invalid_indexes() {
    let mut sidebar = Sidebar::new(&["a", "b"]);
    sidebar.set_selected(5);
    assert_eq!(sidebar.selected(), 0);
    sidebar.set_selected(1);
    assert_eq!(sidebar.selected(), 1);
}

#[test]
fn empty_sidebar_renders_nothing() {
    assert_eq!(Sidebar::new(&[]).render(), "");
}

#[test]
fn tabs_add_appends_and_returns_the_first_index() {
    let mut tabs = Tabs::new(&["a", "b"]);
    assert_eq!(tabs.add(&["c", "d"]), Some(2));
    assert_eq!(tabs.items(), ["a", "b", "c", "d"]);
    assert_eq!(tabs.add(&[]), None);
}

#[test]
fn tabs_remove_returns_the_label_or_empty_when_invalid() {
    let mut tabs = Tabs::new(&["a", "b", "c", "d"]);
    assert_eq!(tabs.remove(0), "a");
    assert_eq!(tabs.items(), ["b", "c", "d"]);
    assert_eq!(tabs.remove(7), "");
    assert_eq!(tabs.items(), ["b", "c", "d"]);
}

#[test]
fn tabs_remove_before_the_active_tab_keeps_the_same_tab() {
    let mut tabs = Tabs::new(&["a", "b", "c"]);
    tabs.set_active(2);
    tabs.remove(0);
    assert_eq!(tabs.active(), 1);
    assert_eq!(tabs.items()[tabs.active()], "c");
}

#[test]
fn tabs_remove_of_the_active_tab_moves_to_the_next() {
    let mut tabs = Tabs::new(&["a", "b", "c"]);
    tabs.set_active(1);
    tabs.remove(1);
    assert_eq!(tabs.active(), 1);
    assert_eq!(tabs.items()[tabs.active()], "c");
}

#[test]
fn tabs_remove_of_the_last_active_tab_moves_back() {
    let mut tabs = Tabs::new(&["a", "b"]);
    tabs.set_active(1);
    tabs.remove(1);
    assert_eq!(tabs.active(), 0);
    assert_eq!(tabs.items()[tabs.active()], "a");
}

#[test]
fn empty_tabs_render_nothing() {
    assert_eq!(Tabs::new(&[]).render(), "");
}

#[test]
fn breadcrumb_add_and_remove() {
    let mut breadcrumb = Breadcrumb::new(&["Inicio"]);
    assert_eq!(breadcrumb.add(&["Config", "Perfil"]), Some(1));
    assert_eq!(breadcrumb.items().len(), 3);

    assert_eq!(breadcrumb.remove(1), "Config");
    assert_eq!(breadcrumb.items(), ["Inicio", "Perfil"]);
    assert_eq!(breadcrumb.remove(9), "");

    breadcrumb.push("Extra");
    assert_eq!(breadcrumb.pop(), "Extra");
}

#[test]
fn breadcrumb_add_empty_returns_none() {
    let mut breadcrumb = Breadcrumb::new(&["Inicio"]);
    assert_eq!(breadcrumb.add(&[]), None);
}

#[test]
fn empty_breadcrumb_renders_nothing() {
    assert_eq!(Breadcrumb::new(&[]).render(), "");
}

#[test]
fn defaults_match_the_go_original() {
    assert_eq!(Tabs::new(&["a"]).separator, " | ");
    assert_eq!(Breadcrumb::new(&["a"]).separator, " > ");

    let sidebar = Sidebar::new(&["a"]);
    assert_eq!(sidebar.width, 20);
    assert_eq!(sidebar.cursor, "\u{25B8} ");
    assert_eq!(sidebar.selected(), 0);
}

#[test]
fn tabs_next_and_previous_wrap_around() {
    let mut tabs = Tabs::new(&["a", "b", "c"]);
    tabs.previous();
    assert_eq!(tabs.active(), 2);
    tabs.next();
    assert_eq!(tabs.active(), 0);
}

#[test]
fn sidebar_next_and_previous_wrap_around() {
    let mut sidebar = Sidebar::new(&["a", "b"]);
    sidebar.previous();
    assert_eq!(sidebar.selected(), 1);
    sidebar.next();
    assert_eq!(sidebar.selected(), 0);
}

#[test]
fn next_and_previous_on_empty_components_do_nothing() {
    let mut tabs = Tabs::new(&[]);
    tabs.next();
    tabs.previous();
    assert_eq!(tabs.active(), 0);

    let mut sidebar = Sidebar::new(&[]);
    sidebar.next();
    sidebar.previous();
    assert_eq!(sidebar.selected(), 0);
}

#[test]
fn tabs_render_joins_the_labels_with_the_separator() {
    let tabs = Tabs::new(&["Home", "Settings", "About"]);
    assert_eq!(strip_ansi(&tabs.render()), "Home | Settings | About");
}

#[test]
fn breadcrumb_render_joins_the_items_with_the_separator() {
    let breadcrumb = Breadcrumb::new(&["Home", "Settings", "Profile"]);
    assert_eq!(
        strip_ansi(&breadcrumb.render()),
        "Home > Settings > Profile"
    );
}

#[test]
fn sidebar_marks_the_selected_item_with_the_cursor() {
    let mut sidebar = Sidebar::new(&["Dashboard", "Users"]);
    sidebar.set_selected(1);

    let rendered = strip_ansi(&sidebar.render());
    let lines: Vec<&str> = rendered.lines().collect();
    assert!(lines[0].contains("  Dashboard"), "linha: {:?}", lines[0]);
    assert!(lines[1].contains("\u{25B8} Users"), "linha: {:?}", lines[1]);
}

#[test]
fn breadcrumb_pop_on_empty_returns_an_empty_string() {
    let mut breadcrumb = Breadcrumb::new(&[]);
    assert_eq!(breadcrumb.pop(), "");
}
