use std::collections::HashMap;
use std::fmt;
use std::io::Write;

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

/// A multi-step wizard with shared state and forward/backward navigation.
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
    current_step: usize,
    data: DataMap,
    title: String,
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
            .finish()
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
            current_step: 0,
            data: DataMap::new(),
            title: "Wizard".to_owned(),
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

    /// Moves to the previous step.
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
        self.current_step -= 1;
        true
    }

    /// Moves to the next step.
    ///
    /// Returns `false` and does not move when already on the last step.
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
        if self.current_step + 1 >= self.steps.len() {
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

    /// Runs every step in order from the first one, printing the title,
    /// progress and completion message to standard output.
    ///
    /// Stops at the first failing step and returns its error.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoSteps`] when the wizard is empty, [`Error::StepFailed`]
    /// when a step fails, and [`Error::Io`] when writing output fails.
    pub fn run(&mut self) -> Result<(), Error> {
        self.run_to(&mut std::io::stdout())
    }

    /// Like [`Wizard::run`], but writes the output to `out`.
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
    /// assert!(String::from_utf8_lossy(&out).contains("completed successfully"));
    /// # Ok::<(), charm_wizard::Error>(())
    /// ```
    pub fn run_to<W: Write>(&mut self, out: &mut W) -> Result<(), Error> {
        if self.steps.is_empty() {
            return Err(Error::NoSteps);
        }

        writeln!(out, "{}", render_title(&self.title))?;
        writeln!(out)?;

        for index in 0..self.steps.len() {
            self.current_step = index;
            writeln!(out, "{}", render_progress(&self.steps, index))?;
            writeln!(out)?;
            self.execute(index)?;
            writeln!(out)?;
        }

        writeln!(out, "{}", render_complete(&self.title))?;
        Ok(())
    }

    /// Runs only the current step, without advancing.
    ///
    /// Useful for interactive wizards that drive navigation themselves.
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
    /// wizard.run_step()?;
    /// assert_eq!(wizard.data()["ran"], true);
    /// # Ok::<(), charm_wizard::Error>(())
    /// ```
    pub fn run_step(&mut self) -> Result<(), Error> {
        if self.steps.is_empty() {
            return Err(Error::NoSteps);
        }
        let index = self.current_step;
        self.execute(index)
    }

    fn execute(&mut self, index: usize) -> Result<(), Error> {
        let total = self.steps.len();
        let step = self
            .steps
            .get_mut(index)
            .ok_or(Error::InvalidIndex { index, total })?;
        (step.run_func)(&mut self.data).map_err(|source| Error::StepFailed {
            name: step.name.clone(),
            source: Box::new(source),
        })
    }
}
