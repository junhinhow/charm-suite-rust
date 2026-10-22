//! Insta snapshot assertion helper.

use charmed_bubbletea::Model;

use crate::{Simulator, ViewSnapshot};

/// Compare the ANSI-stripped view snapshot with an Insta snapshot.
///
/// Set `INSTA_UPDATE=always` to update snapshots using Insta's standard workflow.
///
/// ```rust
/// use charmed_bubbletea::{Cmd, Message, Model};
/// use charm_test::{Simulator, require_snapshot};
///
/// struct StaticModel;
/// impl Model for StaticModel {
///     fn init(&self) -> Option<Cmd> { None }
///     fn update(&mut self, _: Message) -> Option<Cmd> { None }
///     fn view(&self) -> String { "snapshot demo".to_owned() }
/// }
///
/// let sim = Simulator::new(StaticModel);
/// assert_eq!(sim.view(), "snapshot demo");
/// // In a test, call require_snapshot(&sim, Some("static_view"));
/// ```
pub fn require_snapshot<M: Model>(simulator: &Simulator<M>, name: Option<&str>) {
    if let Some(error) = simulator.error() {
        panic!("simulation failed: {error}");
    }
    let mut snapshot = simulator.view_snapshot();
    snapshot.content =
        charm_core::strip_ansi(&snapshot.content.replace("\r\n", "\n").replace('\r', "\n"));
    let rendered = ViewSnapshot::to_string(&snapshot);
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path("../tests/snapshots");
    settings.bind(|| match name {
        Some(name) => insta::assert_snapshot!(name, rendered),
        None => insta::assert_snapshot!(rendered),
    });
}
