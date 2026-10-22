//! # Charm Suite (Rust)
//!
//! Agregador da Charm Suite Rust — um ecossistema curado de bibliotecas para
//! interfaces de terminal (TUIs) no stack Charmbracelet.
//!
//! Cada biblioteca tambem pode ser usada individualmente:
//!
//! ```toml
//! [dependencies]
//! charm-color = "0.1"
//! ```
//!
//! Com o agregador, todas as bibliotecas sao habilitadas por padrao pela
//! feature `full` (ou voce pode selecionar apenas as que precisa com features
//! individuais).
//!
//! ```toml
//! [dependencies]
//! charm-suite = { version = "0.1", features = ["color", "layout", "diff"] }
//! ```
//!
//! ## Bibliotecas
//!
//! | Crate | Descricao |
//! |---|---|
//! | [`charm-banner`] | Gerador de banners ASCII art estilizados |
//! | [`charm-charts`] | Sparklines, barras e barras de progresso |
//! | [`charm-color`] | Utilitarios de cor: escurecer, clarear, misturar, contraste |
//! | [`charm-devtools`] | Inspetor DevTools para Bubble Tea |
//! | [`charm-dialog`] | Dialogos modais (confirm, alert, input, custom) |
//! | [`charm-diff`] | Visualizador de diff (unificado e lado a lado) |
//! | [`charm-form`] | Gerador declarativo de formularios em YAML |
//! | [`charm-i18n`] | Internacionalizacao: catalogos, locale, RTL |
//! | [`charm-layout`] | Motor de layout Flexbox/Grid |
//! | [`charm-nav`] | Navegacao: abas, breadcrumb, sidebar |
//! | [`charm-notify`] | Notificacoes desktop cross-platform |
//! | [`charm-palette`] | Visualizador/gerador de paletas de cor |
//! | [`charm-spinner`] | 30+ animacoes de spinner |
//! | [`charm-test`] | Framework de testes para TUIs |
//! | [`charm-themes`] | Sistema de temas unificado (5 temas) |
//! | [`charm-toast`] | Toasts auto-dismissing |
//! | [`charm-wizard`] | Framework de wizard multi-etapas |
//!
//! O binario [`charm-table`](https://crates.io/crates/charm-table-cli) (crate
//! `charm-table-cli`) e publicado separadamente.
//!
//! [`charm-banner`]: https://docs.rs/charm-banner
//! [`charm-charts`]: https://docs.rs/charm-charts
//! [`charm-color`]: https://docs.rs/charm-color
//! [`charm-devtools`]: https://docs.rs/charm-devtools
//! [`charm-dialog`]: https://docs.rs/charm-dialog
//! [`charm-diff`]: https://docs.rs/charm-diff
//! [`charm-form`]: https://docs.rs/charm-form
//! [`charm-i18n`]: https://docs.rs/charm-i18n
//! [`charm-layout`]: https://docs.rs/charm-layout
//! [`charm-nav`]: https://docs.rs/charm-nav
//! [`charm-notify`]: https://docs.rs/charm-notify
//! [`charm-palette`]: https://docs.rs/charm-palette
//! [`charm-spinner`]: https://docs.rs/charm-spinner
//! [`charm-test`]: https://docs.rs/charm-test
//! [`charm-themes`]: https://docs.rs/charm-themes
//! [`charm-toast`]: https://docs.rs/charm-toast
//! [`charm-wizard`]: https://docs.rs/charm-wizard

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "banner")]
pub use charm_banner as banner;
#[cfg(feature = "charts")]
pub use charm_charts as charts;
#[cfg(feature = "color")]
pub use charm_color as color;
#[cfg(feature = "devtools")]
pub use charm_devtools as devtools;
#[cfg(feature = "dialog")]
pub use charm_dialog as dialog;
#[cfg(feature = "diff")]
pub use charm_diff as diff;
#[cfg(feature = "form")]
pub use charm_form as form;
#[cfg(feature = "i18n")]
pub use charm_i18n as i18n;
#[cfg(feature = "layout")]
pub use charm_layout as layout;
#[cfg(feature = "nav")]
pub use charm_nav as nav;
#[cfg(feature = "notify")]
pub use charm_notify as notify;
#[cfg(feature = "palette")]
pub use charm_palette as palette;
#[cfg(feature = "spinner")]
pub use charm_spinner as spinner;
#[cfg(feature = "test")]
pub use charm_test as test;
#[cfg(feature = "themes")]
pub use charm_themes as themes;
#[cfg(feature = "toast")]
pub use charm_toast as toast;
#[cfg(feature = "wizard")]
pub use charm_wizard as wizard;
