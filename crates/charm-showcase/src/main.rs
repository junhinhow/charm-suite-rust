#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Render a deterministic, non-interactive overview of the Charm Suite crates.

use std::collections::BTreeMap;
use std::error::Error;
use std::time::Duration;

use charm_charts::{ProgressBar, Sparkline};
use charm_core::strip_ansi;
use charm_dialog::Dialog;
use charm_i18n::Catalog;
use charm_layout::{Direction, Flex};
use charm_nav::{Sidebar, Tabs};
use charm_themes::themes;
use charm_toast::{Toast, ToastType};
use charm_wizard::Wizard;

fn main() -> Result<(), Box<dyn Error>> {
    let theme = themes::dracula();
    let title = theme.lipgloss().title.render("CharmSuite Showcase");
    let banner = charm_banner::render("CHARM SUITE", &[]);

    let mut tabs = Tabs::new(&["Overview", "Components", "Form"]);
    tabs.set_active(1);
    let sidebar = Sidebar::new(&["Dashboard", "Charts", "Settings"]).render();
    let layout = Flex::new(Direction::Row)
        .set_size(64, 4)
        .set_gap(1)
        .add_child(0.25, sidebar)
        .add_child(
            0.75,
            format!("{}\n\n{}", tabs.render(), "All systems ready"),
        )
        .render();

    let sparkline = Sparkline::new(&[2.0, 5.0, 3.0, 8.0, 6.0, 9.0])
        .width(16)
        .render();
    let progress = ProgressBar::new(0.72).width(18).label("Build").render();

    let spinner = charm_spinner::dots();
    let spinner_frames = (0..3)
        .map(|frame| spinner.render(frame))
        .collect::<Vec<_>>()
        .join(" ");

    let diff = charm_diff::unified_diff(
        "theme = \"nord\"\nstatus = \"idle\"\n",
        "theme = \"dracula\"\nstatus = \"ready\"\n",
    );
    let dialog = Dialog::new_confirm("Deploy demo?", "Apply the sample configuration?").render(0);
    let toast = Toast::new("Configuration saved", Duration::from_secs(5))
        .with_type(ToastType::Success)
        .render(0);

    let colors = charm_color::palette(theme.primary, 4);
    let palette = charm_palette::show_palette(&colors, 36);

    let mut catalog = Catalog::new();
    catalog.add_locale(
        "en",
        BTreeMap::from([
            ("dashboard".to_owned(), "Dashboard".to_owned()),
            ("ready".to_owned(), "Ready".to_owned()),
        ]),
    );
    catalog.add_locale(
        "pt-BR",
        BTreeMap::from([
            ("dashboard".to_owned(), "Painel".to_owned()),
            ("ready".to_owned(), "Pronto".to_owned()),
        ]),
    );
    let english = catalog.t("dashboard").to_owned();
    catalog.set_locale("pt-BR");
    let portuguese = catalog.t("dashboard");
    let translations = format!("en={english} | pt-BR={portuguese}");

    let mut wizard = Wizard::new()
        .with_title("Three-step setup")
        .add_step("Theme", "Select the Dracula theme", |data| {
            data.insert("theme".to_owned(), serde_json::json!("dracula"));
            Ok(())
        })
        .add_step("Layout", "Prepare the dashboard", |data| {
            data.insert("layout".to_owned(), serde_json::json!("sidebar + content"));
            Ok(())
        })
        .add_step("Finish", "Mark the demo ready", |data| {
            data.insert("ready".to_owned(), serde_json::json!(true));
            Ok(())
        });
    let mut wizard_bytes = Vec::new();
    wizard.run_to(&mut wizard_bytes)?;
    let wizard_output = String::from_utf8(wizard_bytes)?;

    let form = charm_form::load_from_bytes(include_str!("demo-form.yaml").as_bytes())?;
    let form_json = charm_form::to_json(&form)?;

    let screen = format!(
        "=== CharmSuite | Unified Demo ===\n{}\n{}\nTheme: {} ({})\n\n[Navigation + Flex Row]\n{}\n\n[Charts]\nSparkline: {}\nProgress: {}\n\n[Spinner: three deterministic frames]\n{}\n\n[Diff]\n{}\n[Confirmation Dialog]\n{}\n[Toast]\n{}\n\n[Color Palette]\n{}\n\n[Translations]\n{}\n\n[Executed Wizard]\n{}\n[Embedded YAML Form -> JSON]\n{}",
        banner,
        title,
        theme.name,
        theme.primary.to_hex(),
        layout,
        sparkline,
        progress,
        spinner_frames,
        diff,
        dialog,
        toast,
        palette,
        translations,
        wizard_output,
        form_json,
    );

    println!("{}", strip_ansi(&screen));
    Ok(())
}
