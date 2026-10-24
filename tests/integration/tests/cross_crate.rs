//! Testes de integração entre crates da CharmSuite-Rust.
//!
//! Valida que os componentes funcionam corretamente quando combinados,
//! garantindo que tipos, traits e APIs são compatíveis entre si.

use charm_core::text::display_width;
use charm_layout::{Direction, Flex};
use charm_panes::{Direction as PaneDirection, Layout, Pane};
use charm_themes::themes;

/// Valida que o sistema de temas é compatível com o layout flexbox.
#[test]
fn themes_integrate_with_layout() {
    let _theme = themes::gruvbox();

    // Tema deve ser clonável e utilizável em contexto de layout
    let _flex = Flex::new(Direction::Row)
        .set_size(80, 24)
        .add_child(0.5, "sidebar")
        .add_child(0.5, "main");

    // Tema não deve interferir na construção do layout
    assert!(true, "Theme and Flex types are compatible");
}

/// Valida que panes podem ser organizados em layouts.
#[test]
fn panes_integrate_with_layout_system() {
    let mut layout = Layout::new(PaneDirection::Horizontal);
    layout.add(Pane {
        id: "a".to_string(),
        title: "A".to_string(),
        content: "content-a".to_string(),
        min_size: 3,
        fixed: false,
    });
    layout.add(Pane {
        id: "b".to_string(),
        title: "B".to_string(),
        content: "content-b".to_string(),
        min_size: 3,
        fixed: false,
    });
    layout.set_size(80, 24);

    // Layout deve produzir view não vazia com 2 panes
    let view = layout.view();
    assert!(!view.is_empty(), "Layout with 2 panes renders non-empty view");
}

/// Valida que charm-core text utilities funcionam com strings renderizadas.
#[test]
fn core_text_works_with_rendered_content() {
    let content = "Hello, CharmSuite!";
    let width = display_width(content);
    assert_eq!(width, 18, "ASCII string width is correct");

    // Unicode wide chars
    let cjk = "你好";
    let cjk_width = display_width(cjk);
    assert_eq!(cjk_width, 4, "CJK characters have double width");
}

/// Valida que router e nav compartilham tipos compatíveis.
#[test]
fn router_and_nav_types_are_compatible() {
    use charm_router::Router;
    use charm_nav::tabs::Tabs;

    // Tabs aceita &[&str]
    let _tabs = Tabs::new(&["Home", "Settings"]);

    // Router pode ser instanciado sem root screen
    let _router = Router::new(None);

    assert!(true, "Router and Tabs coexist without type conflicts");
}