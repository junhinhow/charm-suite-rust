use std::collections::HashMap;
use std::fmt;
use std::io::{IsTerminal, Write};

use serde_json::Value;

use crate::progress::{render_complete, render_progress, render_title};
use crate::Error;

/// Shared state passed to every step of a [`Wizard`].
pub type DataMap = HashMap<String, Value>;

/// Callback executed by a [`Step`]. It receives the shared [`DataMap`].
pub type StepFn = Box<dyn FnMut(&mut DataMap) -> Result<(), Error>>;

/// A single step of a [`Wizard`].
pub struct Step {
    /// Short identifier of the step, such as `"config"` or `"deploy"`.
    pub name: String,
    /// Human-readable description shown while the step is active.
    pub description: String,
    /// Logic executed by the step.
    pub run_func: StepFn,
}

impl fmt::Debug for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Step")
            .field("name", &self.name)
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

/// Outcome of the last execution of a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepStatus {
    /// Not executed yet, or undone by [`Wizard::back`].
    Pending,
    /// Executed successfully.
    Completed,
    /// Returned an error. The wizard refuses to move past it.
    Failed,
}

/// A multi-step wizard with shared state and forward/backward navigation.
///
/// Going back undoes the data changes of the step being left: the data map
/// is restored to the state it had before that step ran.
///
/// # Examples
///
/// ```
/// use charm_wizard::{DataMap, Error, Wizard};
/// use serde_json::json;
///
/// let mut wizard = Wizard::new()
///     .with_title("Setup")
///     .add_step("config", "Configuration", |data: &mut DataMap| {
///         data.insert("project".into(), json!("charm"));
///         Ok(())
///     })
///     .add_step("check", "Validation", |data: &mut DataMap| {
///         if data.contains_key("project") {
///             Ok(())
///         } else {
///             Err(Error::custom("project is missing"))
///         }
///     });
///
/// wizard.run_to(&mut Vec::<u8>::new())?;
/// assert_eq!(wizard.data()["project"], "charm");
/// # Ok::<(), Error>(())
/// ```
pub struct Wizard {
    steps: Vec<Step>,
    statuses: Vec<StepStatus>,
    current_step: usize,
    data: DataMap,
    title: String,
    /// Copy of the data taken right before each step ran, used by `back`.
    before_state: HashMap<usize, DataMap>,
}

impl Default for Wizard {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Wizard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wizard")
            .field("steps", &self.steps)
            .field("current_step", &self.current_step)
            .field("data", &self.data)
            .field("title", &self.title)
            .finish_non_exhaustive()
    }
}

impl Wizard {
    /// Creates an empty wizard titled `"Wizard"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::Wizard;
    ///
    /// let wizard = Wizard::new();
    /// assert_eq!(wizard.total_steps(), 0);
    /// assert_eq!(wizard.current_step(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            statuses: Vec::new(),
            current_step: 0,
            data: DataMap::new(),
            title: "Wizard".to_owned(),
            before_state: HashMap::new(),
        }
    }

    /// Sets the title shown when the wizard runs.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::Wizard;
    ///
    /// let wizard = Wizard::new().with_title("Installer");
    /// assert_eq!(wizard.total_steps(), 0);
    /// ```
    #[must_use]
    pub fn with_title(mut self, title: &str) -> Self {
        title.clone_into(&mut self.title);
        self
    }

    /// Appends a step. The closure receives the shared data and may fail with
    /// an [`Error`] to stop the wizard.
    ///
    /// Use a closure that does nothing, such as `|_| Ok(())`, for a step
    /// without work. The Go API accepts a nil function for this.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::{DataMap, Wizard};
    ///
    /// let wizard = Wizard::new().add_step("noop", "Does nothing", |_: &mut DataMap| Ok(()));
    /// assert_eq!(wizard.total_steps(), 1);
    /// ```
    #[must_use]
    pub fn add_step(
        mut self,
        name: &str,
        description: &str,
        f: impl FnMut(&mut DataMap) -> Result<(), Error> + 'static,
    ) -> Self {
        self.steps.push(Step {
            name: name.to_owned(),
            description: description.to_owned(),
            run_func: Box::new(f),
        });
        self.statuses.push(StepStatus::Pending);
        self
    }

    /// Returns the shared data map.
    pub fn data(&self) -> &DataMap {
        &self.data
    }

    /// Inserts a value into the shared data map.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::Wizard;
    /// use serde_json::json;
    ///
    /// let wizard = Wizard::new().set_data("region", json!("eu"));
    /// assert_eq!(wizard.data()["region"], "eu");
    /// ```
    #[must_use]
    pub fn set_data(mut self, key: &str, value: Value) -> Self {
        self.data.insert(key.to_owned(), value);
        self
    }

    /// Returns the value of `key` when it is a string.
    ///
    /// Returns `None` when the key is missing or holds another type.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::Wizard;
    /// use serde_json::json;
    ///
    /// let wizard = Wizard::new().set_data("name", json!("ana"));
    /// assert_eq!(wizard.get_string("name"), Some("ana"));
    /// assert_eq!(wizard.get_string("missing"), None);
    /// ```
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.data.get(key)?.as_str()
    }

    /// Returns the value of `key` when it is a boolean.
    ///
    /// Returns `None` when the key is missing or holds another type.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::Wizard;
    /// use serde_json::json;
    ///
    /// let wizard = Wizard::new().set_data("active", json!(true));
    /// assert_eq!(wizard.get_bool("active"), Some(true));
    /// ```
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }

    /// Returns the value of `key` when it is an integer.
    ///
    /// Returns `None` when the key is missing, holds another type, or holds a
    /// number with a fractional part. The Go API uses `int`; the Rust port
    /// returns `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::Wizard;
    /// use serde_json::json;
    ///
    /// let wizard = Wizard::new().set_data("count", json!(3));
    /// assert_eq!(wizard.get_int("count"), Some(3));
    /// ```
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_i64()
    }

    /// Returns the index of the current step (zero-based).
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Returns the number of registered steps.
    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }

    /// Returns the registered steps in order.
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Returns the name of the current step, or `""` when there are no steps.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::{DataMap, Wizard};
    ///
    /// let wizard = Wizard::new().add_step("config", "Setup", |_: &mut DataMap| Ok(()));
    /// assert_eq!(wizard.current_step_name(), "config");
    /// ```
    pub fn current_step_name(&self) -> &str {
        self.steps
            .get(self.current_step)
            .map_or("", |step| step.name.as_str())
    }

    /// Returns `true` when the current step is the first one.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::Wizard;
    ///
    /// assert!(Wizard::new().is_first());
    /// ```
    pub fn is_first(&self) -> bool {
        self.current_step == 0
    }

    /// Returns `true` when the current step is the last one, or when there are
    /// no steps.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::Wizard;
    ///
    /// assert!(Wizard::new().is_last());
    /// ```
    pub fn is_last(&self) -> bool {
        self.steps.is_empty() || self.current_step + 1 >= self.steps.len()
    }

    /// Returns the fraction of steps completed successfully, from `0.0` to `1.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::{DataMap, Wizard};
    ///
    /// let mut wizard = Wizard::new()
    ///     .add_step("a", "A", |_: &mut DataMap| Ok(()))
    ///     .add_step("b", "B", |_: &mut DataMap| Ok(()));
    /// wizard.run_step_to(&mut Vec::<u8>::new())?;
    /// assert_eq!(wizard.progress(), 0.5);
    /// # Ok::<(), charm_wizard::Error>(())
    /// ```
    pub fn progress(&self) -> f64 {
        if self.steps.is_empty() {
            return 0.0;
        }
        let completed = self
            .statuses
            .iter()
            .filter(|status| **status == StepStatus::Completed)
            .count();
        let completed = u32::try_from(completed).unwrap_or(u32::MAX);
        let total = u32::try_from(self.steps.len()).unwrap_or(u32::MAX);
        f64::from(completed) / f64::from(total)
    }

    /// Moves to the previous step and undoes the data changes of the step
    /// being left.
    ///
    /// Returns `false` and does not move when already on the first step.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::Wizard;
    ///
    /// let mut wizard = Wizard::new();
    /// assert!(!wizard.back());
    /// ```
    pub fn back(&mut self) -> bool {
        if self.current_step == 0 {
            return false;
        }
        if let Some(snapshot) = self.before_state.get(&self.current_step) {
            self.data = snapshot.clone();
            if let Some(status) = self.statuses.get_mut(self.current_step) {
                *status = StepStatus::Pending;
            }
        }
        self.current_step -= 1;
        true
    }

    /// Moves to the next step.
    ///
    /// Returns `false` and does not move when already on the last step, or
    /// when the current step failed: a broken step cannot be skipped.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::{DataMap, Wizard};
    ///
    /// let mut wizard = Wizard::new()
    ///     .add_step("a", "A", |_: &mut DataMap| Ok(()))
    ///     .add_step("b", "B", |_: &mut DataMap| Ok(()));
    /// assert!(wizard.forward());
    /// assert!(!wizard.forward());
    /// ```
    pub fn forward(&mut self) -> bool {
        if self.is_last() {
            return false;
        }
        if self.statuses.get(self.current_step) == Some(&StepStatus::Failed) {
            return false;
        }
        self.current_step += 1;
        true
    }

    /// Moves to the step at `index`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidIndex`] when `index` is not a registered step.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::{DataMap, Wizard};
    ///
    /// let mut wizard = Wizard::new().add_step("a", "A", |_: &mut DataMap| Ok(()));
    /// assert!(wizard.go_to(0).is_ok());
    /// assert!(wizard.go_to(5).is_err());
    /// ```
    pub fn go_to(&mut self, index: usize) -> Result<(), Error> {
        let total = self.steps.len();
        if index >= total {
            return Err(Error::InvalidIndex { index, total });
        }
        self.current_step = index;
        Ok(())
    }

    /// Runs the steps in order, starting at the current step, and prints the
    /// title, the progress and the completion message to standard output.
    ///
    /// Stops at the first failing step and returns its error. When standard
    /// output is a terminal, each progress block is redrawn in place.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoSteps`] when the wizard is empty, [`Error::StepFailed`]
    /// when a step fails, and [`Error::Io`] when writing output fails.
    pub fn run(&mut self) -> Result<(), Error> {
        let mut out = std::io::stdout();
        let in_place = out.is_terminal();
        self.run_with(&mut out, in_place)
    }

    /// Like [`Wizard::run`], but writes the output to `out`.
    ///
    /// The output is never redrawn in place, because `out` may not be a
    /// terminal.
    ///
    /// # Errors
    ///
    /// Same as [`Wizard::run`].
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::{DataMap, Wizard};
    ///
    /// let mut wizard = Wizard::new().add_step("a", "A", |_: &mut DataMap| Ok(()));
    /// let mut out = Vec::<u8>::new();
    /// wizard.run_to(&mut out)?;
    /// assert!(String::from_utf8_lossy(&out).contains("concluído com sucesso"));
    /// # Ok::<(), charm_wizard::Error>(())
    /// ```
    pub fn run_to<W: Write>(&mut self, out: &mut W) -> Result<(), Error> {
        self.run_with(out, false)
    }

    /// Runs only the current step, without advancing, and prints its progress.
    ///
    /// The progress block is printed even when the step fails, so the user
    /// can see where the wizard stopped. Standard output is used, as in
    /// [`Wizard::run`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoSteps`] when the wizard is empty and
    /// [`Error::StepFailed`] when the step fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::{DataMap, Wizard};
    /// use serde_json::json;
    ///
    /// let mut wizard = Wizard::new().add_step("a", "A", |d: &mut DataMap| {
    ///     d.insert("ran".into(), json!(true));
    ///     Ok(())
    /// });
    /// wizard.run_step_to(&mut Vec::<u8>::new())?;
    /// assert_eq!(wizard.get_bool("ran"), Some(true));
    /// # Ok::<(), charm_wizard::Error>(())
    /// ```
    pub fn run_step(&mut self) -> Result<(), Error> {
        self.run_step_to(&mut std::io::stdout())
    }

    /// Like [`Wizard::run_step`], but writes the progress to `out`.
    ///
    /// # Errors
    ///
    /// Same as [`Wizard::run_step`].
    pub fn run_step_to<W: Write>(&mut self, out: &mut W) -> Result<(), Error> {
        if self.steps.is_empty() {
            return Err(Error::NoSteps);
        }
        let index = self.current_step;
        let outcome = self.execute(index);
        writeln!(out, "{}", render_progress(&self.steps, index))?;
        outcome
    }

    fn run_with<W: Write>(&mut self, out: &mut W, in_place: bool) -> Result<(), Error> {
        if self.steps.is_empty() {
            return Err(Error::NoSteps);
        }

        write!(out, "{}\n\n", render_title(&self.title))?;

        let mut lines = 0;
        for index in self.current_step..self.steps.len() {
            self.current_step = index;

            if in_place && lines > 0 {
                write!(out, "{}", erase_sequence(lines))?;
            }

            // The progress block is printed even when the step fails, so the
            // user knows which step stopped the run.
            let outcome = self.execute(index);
            let block = render_progress(&self.steps, index);
            writeln!(out, "{block}")?;
            lines = count_lines(&block);

            outcome?;
        }

        if in_place && lines > 0 {
            write!(out, "{}", erase_sequence(lines))?;
        }
        writeln!(out, "{}", render_complete(&self.title))?;
        Ok(())
    }

    fn execute(&mut self, index: usize) -> Result<(), Error> {
        let total = self.steps.len();
        if index >= total {
            return Err(Error::InvalidIndex { index, total });
        }

        self.before_state.insert(index, self.data.clone());

        let step = &mut self.steps[index];
        let outcome = (step.run_func)(&mut self.data);
        let status = if outcome.is_ok() {
            StepStatus::Completed
        } else {
            StepStatus::Failed
        };
        if let Some(slot) = self.statuses.get_mut(index) {
            *slot = status;
        }

        outcome.map_err(|source| Error::StepFailed {
            name: step.name.clone(),
            source: Box::new(source),
        })
    }
}

/// Escape sequence that erases the last `lines` lines written to a terminal.
///
/// Returns an empty string when `lines` is zero.
pub(crate) fn erase_sequence(lines: usize) -> String {
    if lines == 0 {
        String::new()
    } else {
        format!("\x1b[{lines}A\x1b[J")
    }
}

/// Counts the lines of a block of text. An empty text has no lines.
pub(crate) fn count_lines(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.matches('\n').count() + 1
    }
}
