//! Resizable, keyboard-driven panes for terminal user interfaces.
//!
//! This crate is a Rust port of the Go library
//! [`github.com/junhinhow/charm-panes`](https://github.com/junhinhow/charm-panes).
//! A [`Layout`] organises [`Pane`]s horizontally or vertically, sharing the
//! available space and letting the focused pane grow or shrink while the others
//! rebalance without ever crossing their minimum size.
//!
//! ```
//! use charm_panes::{Direction, Layout, Pane};
//!
//! let mut layout = Layout::new(Direction::Horizontal);
//! layout.set_size(60, 8);
//! layout.add(Pane {
//!     id: "sidebar".to_string(),
//!     title: "Files".to_string(),
//!     content: "main.go".to_string(),
//!     min_size: 12,
//!     fixed: true,
//! });
//! layout.add(Pane {
//!     id: "editor".to_string(),
//!     title: "Editor".to_string(),
//!     content: "fn main() {}".to_string(),
//!     min_size: 0,
//!     fixed: false,
//! });
//! assert!(!layout.view().is_empty());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(non_snake_case)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::manual_memcpy,
    clippy::needless_pass_by_value,
    clippy::needless_range_loop,
    clippy::redundant_closure_for_method_calls,
    clippy::uninlined_format_args
)]

use charmed_bubbletea::{Cmd, KeyMsg, Message, WindowSizeMsg};
use lipgloss::Style;

#[cfg(test)]
mod tests;

/// Layout axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Panes are placed side by side; the main dimension is the width.
    #[default]
    Horizontal,
    /// Panes are stacked; the main dimension is the height.
    Vertical,
}

/// A single pane of a [`Layout`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pane {
    /// Unique identifier of the pane inside the layout.
    pub id: String,
    /// Text shown on the title row.
    pub title: String,
    /// Textual content of the pane.
    pub content: String,
    /// Minimum size in columns ([`Direction::Horizontal`]) or rows
    /// ([`Direction::Vertical`]). Values `<= 0` are normalised to `3`.
    pub min_size: i32,
    /// When set, the pane does not donate space when another pane is resized.
    pub fixed: bool,
}

/// A value returned by [`Layout::view_model`], mirroring Bubble Tea's `View`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct View {
    /// Rendered contents of the layout.
    pub content: String,
}

const DEFAULT_MIN: i32 = 3;
const MARKER_FOCUSED: &str = "▌ ";
const MARKER_UNFOCUSED: &str = "  ";
const DIVIDER_VERTICAL: &str = "│";
const DIVIDER_HORIZONTAL: &str = "─";

/// An ordered set of panes with focus and resizable sizes.
#[derive(Debug, Clone)]
pub struct Layout {
    dir: Direction,
    panes: Vec<Pane>,
    sizes: Vec<i32>,
    focus: i32,
    width: i32,
    height: i32,
    dirty: bool,
}

impl Layout {
    /// Creates an empty layout in the given direction.
    ///
    /// Unknown directions are treated as [`Direction::Horizontal`]. The layout
    /// starts at `80x24` until [`Layout::set_size`] is called.
    pub fn new(dir: Direction) -> Self {
        let dir = if dir == Direction::Vertical {
            Direction::Vertical
        } else {
            Direction::Horizontal
        };
        Self {
            dir,
            panes: Vec::new(),
            sizes: Vec::new(),
            focus: -1,
            width: 80,
            height: 24,
            dirty: false,
        }
    }

    /// Go-style alias for [`Layout::new`].
    pub fn New(dir: Direction) -> Self {
        Self::new(dir)
    }

    /// Adds a pane to the end of the layout.
    ///
    /// A pane with an already existing id replaces the previous one in place.
    pub fn add(&mut self, mut p: Pane) {
        p.min_size = normalize_min(p.min_size);
        if let Some(i) = self.index(&p.id) {
            self.panes[i] = p;
            self.fit();
            return;
        }
        let target = self.average_size();
        self.panes.push(p);
        self.sizes.push(target);
        if self.focus < 0 {
            self.focus = 0;
        }
        if self.dirty {
            self.fit();
            return;
        }
        self.reset_sizes();
    }

    /// Go-style alias for [`Layout::add`].
    pub fn Add(&mut self, p: Pane) {
        self.add(p);
    }

    /// Removes the pane with the given id, returning whether it existed.
    pub fn remove(&mut self, id: &str) -> bool {
        let Some(i) = self.index(id) else {
            return false;
        };
        self.panes.remove(i);
        self.sizes.remove(i);
        if self.panes.is_empty() {
            self.focus = -1;
        } else if self.focus > i as i32 {
            self.focus -= 1;
        } else if self.focus == i as i32 && self.focus >= self.panes.len() as i32 {
            self.focus = self.panes.len() as i32 - 1;
        }
        self.fit();
        true
    }

    /// Go-style alias for [`Layout::remove`].
    pub fn Remove(&mut self, id: &str) -> bool {
        self.remove(id)
    }

    /// Removes the pane with the given id, ignoring the result.
    pub fn idrop(&mut self, id: &str) {
        self.remove(id);
    }

    /// Go-style alias for [`Layout::idrop`].
    pub fn IDrop(&mut self, id: &str) {
        self.idrop(id);
    }

    /// Snake-case alias for [`Layout::idrop`].
    pub fn i_drop(&mut self, id: &str) {
        self.idrop(id);
    }

    /// Returns a copy of the pane with the given id, if it exists.
    pub fn pane(&self, id: &str) -> (Pane, bool) {
        if let Some(i) = self.index(id) {
            (self.panes[i].clone(), true)
        } else {
            (Pane::default(), false)
        }
    }

    /// Go-style alias for [`Layout::pane`].
    pub fn Pane(&self, id: &str) -> (Pane, bool) {
        self.pane(id)
    }

    /// Returns a copy of the pane with the given id, if it exists.
    pub fn get_pane(&self, id: &str) -> Option<Pane> {
        self.index(id).map(|i| self.panes[i].clone())
    }

    /// Replaces the content of the pane with the given id.
    pub fn set_content(&mut self, id: &str, content: &str) {
        if let Some(i) = self.index(id) {
            self.panes[i].content = content.to_string();
        }
    }

    /// Go-style alias for [`Layout::set_content`].
    pub fn SetContent(&mut self, id: &str, content: &str) {
        self.set_content(id, content);
    }

    /// Replaces the title of the pane with the given id.
    pub fn set_title(&mut self, id: &str, title: &str) {
        if let Some(i) = self.index(id) {
            self.panes[i].title = title.to_string();
        }
    }

    /// Go-style alias for [`Layout::set_title`].
    pub fn SetTitle(&mut self, id: &str, title: &str) {
        self.set_title(id, title);
    }

    /// Returns the ids of the panes in layout order.
    pub fn ids(&self) -> Vec<String> {
        self.panes.iter().map(|p| p.id.clone()).collect()
    }

    /// Go-style alias for [`Layout::ids`].
    pub fn IDs(&self) -> Vec<String> {
        self.ids()
    }

    /// Sets the available width and height.
    ///
    /// Negative values are treated as zero.
    pub fn set_size(&mut self, width: i32, height: i32) {
        self.width = width.max(0);
        self.height = height.max(0);
        self.fit();
    }

    /// Go-style alias for [`Layout::set_size`].
    pub fn SetSize(&mut self, width: i32, height: i32) {
        self.set_size(width, height);
    }

    /// Returns the available width and height.
    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    /// Go-style alias for [`Layout::size`].
    pub fn Size(&self) -> (i32, i32) {
        self.size()
    }

    /// Moves the focus to the given index, clamped to the valid range.
    pub fn focus(&mut self, index: i32) {
        if self.panes.is_empty() {
            self.focus = -1;
            return;
        }
        self.focus = index.clamp(0, self.panes.len() as i32 - 1);
    }

    /// Go-style alias for [`Layout::focus`].
    pub fn Focus(&mut self, index: i32) {
        self.focus(index);
    }

    /// Returns the index of the focused pane, or `-1` when empty.
    pub fn focused(&self) -> i32 {
        self.focus
    }

    /// Go-style alias for [`Layout::focused`].
    pub fn Focused(&self) -> i32 {
        self.focused()
    }

    /// Moves the focus to the next pane, wrapping around.
    pub fn focus_next(&mut self) {
        if self.panes.is_empty() {
            return;
        }
        if self.focus < 0 {
            self.focus = 0;
            return;
        }
        self.focus = (self.focus + 1) % self.panes.len() as i32;
    }

    /// Go-style alias for [`Layout::focus_next`].
    pub fn FocusNext(&mut self) {
        self.focus_next();
    }

    /// Moves the focus to the previous pane, wrapping around.
    pub fn focus_prev(&mut self) {
        if self.panes.is_empty() {
            return;
        }
        if self.focus < 0 {
            self.focus = 0;
            return;
        }
        self.focus = (self.focus - 1 + self.panes.len() as i32) % self.panes.len() as i32;
    }

    /// Go-style alias for [`Layout::focus_prev`].
    pub fn FocusPrev(&mut self) {
        self.focus_prev();
    }

    /// Bubble Tea integration: performs no initial command.
    pub fn init(&self) -> Option<Cmd> {
        None
    }

    /// Go-style alias for [`Layout::init`].
    pub fn Init(&self) -> Option<Cmd> {
        self.init()
    }

    /// Bubble Tea integration: handles window size and key messages.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
            self.set_size(i32::from(size.width), i32::from(size.height));
        } else if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            self.handle_key(key);
        }
        None
    }

    /// Go-style alias for [`Layout::update`].
    pub fn Update(&mut self, msg: Message) -> Option<Cmd> {
        self.update(msg)
    }

    /// Returns the layout rendered as a Bubble Tea `View`.
    pub fn view_model(&self) -> View {
        View {
            content: self.view(),
        }
    }

    /// Go-style alias for [`Layout::view_model`].
    pub fn ViewModel(&self) -> View {
        self.view_model()
    }

    /// Renders the layout to a string.
    pub fn view(&self) -> String {
        if self.panes.is_empty() || self.width <= 0 || self.height <= 0 {
            return String::new();
        }
        let visible = self.visible_count(self.main_size());
        if visible == 0 {
            return String::new();
        }
        if self.panes.len() == 1 {
            return render_content(&self.panes[0].content, self.width, self.height);
        }
        if self.dir == Direction::Vertical {
            self.render_vertical(visible)
        } else {
            self.render_horizontal(visible)
        }
    }

    /// Go-style alias for [`Layout::view`].
    pub fn View(&self) -> String {
        self.view()
    }

    /// Increases or decreases the focused pane by `delta`.
    pub fn resize(&mut self, delta: i32) {
        if delta == 0 {
            return;
        }
        self.fit();
        let visible = self.visible_count(self.main_size());
        if visible == 0 {
            return;
        }
        let focus = self.focus;
        if focus < 0 || focus >= visible {
            return;
        }
        if delta > 0 {
            self.grow_focused(focus, delta, visible);
        } else {
            self.shrink_focused(focus, -delta, visible);
        }
    }

    /// Go-style alias for [`Layout::resize`].
    pub fn Resize(&mut self, delta: i32) {
        self.resize(delta);
    }

    /// Sets the focused pane to an exact size.
    pub fn resize_to(&mut self, size: i32) {
        self.fit();
        if self.focus < 0 || self.focus >= self.panes.len() as i32 {
            return;
        }
        let mut size = size;
        if size < self.min_for(self.focus as usize) {
            size = self.min_for(self.focus as usize);
        }
        let current = self.sizes[self.focus as usize];
        if size == current {
            return;
        }
        self.resize(size - current);
    }

    /// Go-style alias for [`Layout::resize_to`].
    pub fn ResizeTo(&mut self, size: i32) {
        self.resize_to(size);
    }

    /// Resets every visible pane to an equal share.
    pub fn reset(&mut self) {
        self.reset_sizes();
        self.dirty = false;
    }

    /// Go-style alias for [`Layout::reset`].
    pub fn Reset(&mut self) {
        self.reset();
    }

    /// Returns the current size of every pane in the main dimension.
    pub fn sizes(&self) -> Vec<i32> {
        let mut out = vec![0; self.sizes.len()];
        for (i, s) in self.sizes.iter().enumerate() {
            out[i] = *s;
        }
        out
    }

    /// Go-style alias for [`Layout::sizes`].
    pub fn Sizes(&self) -> Vec<i32> {
        self.sizes()
    }

    /// Returns how many panes fit in the current dimension.
    pub fn visible(&self) -> i32 {
        self.visible_count(self.main_size())
    }

    /// Go-style alias for [`Layout::visible`].
    pub fn Visible(&self) -> i32 {
        self.visible()
    }

    // ---- internals ----

    fn index(&self, id: &str) -> Option<usize> {
        self.panes.iter().position(|p| p.id == id)
    }

    fn min_for(&self, i: usize) -> i32 {
        normalize_min(self.panes[i].min_size)
    }

    fn main_size(&self) -> i32 {
        if self.dir == Direction::Vertical {
            self.height
        } else {
            self.width
        }
    }

    fn average_size(&self) -> i32 {
        let mut sum = 0;
        let mut n = 0;
        for &s in &self.sizes {
            if s > 0 {
                sum += s;
                n += 1;
            }
        }
        if n == 0 {
            DEFAULT_MIN
        } else {
            (sum / n).max(DEFAULT_MIN)
        }
    }

    fn visible_count(&self, total: i32) -> i32 {
        if total <= 0 {
            return 0;
        }
        let mut used = 0;
        let mut visible = 0;
        for i in 0..self.panes.len() {
            let mut cost = self.min_for(i);
            if i > 0 {
                cost += 1;
            }
            if used + cost > total {
                break;
            }
            used += cost;
            visible += 1;
        }
        visible
    }

    fn number_hidden(&self, visible: i32) -> i32 {
        (self.panes.len() as i32 - visible).max(0)
    }

    fn reset_sizes(&mut self) {
        let n = self.panes.len();
        if n == 0 {
            self.sizes.clear();
            return;
        }
        if self.sizes.len() != n {
            let mut adjusted = vec![0; n];
            for i in 0..self.sizes.len().min(n) {
                adjusted[i] = self.sizes[i];
            }
            self.sizes = adjusted;
        }
        let total = self.main_size();
        let visible = self.visible_count(total);
        for i in 0..n {
            self.sizes[i] = 0;
        }
        if visible == 0 {
            return;
        }
        let available = total - (visible - 1);
        let base = available / visible;
        let rem = available % visible;
        for i in 0..visible as usize {
            self.sizes[i] = base;
            if (i as i32) < rem {
                self.sizes[i] += 1;
            }
        }
        self.correct_mins(visible);
    }

    fn fit(&mut self) {
        let n = self.panes.len();
        if n == 0 {
            self.sizes.clear();
            self.focus = -1;
            return;
        }
        if self.sizes.len() != n {
            let mut adjusted = vec![0; n];
            for i in 0..self.sizes.len().min(n) {
                adjusted[i] = self.sizes[i];
            }
            self.sizes = adjusted;
        }
        let total = self.main_size();
        let visible = self.visible_count(total);
        if visible == 0 {
            for i in 0..n {
                self.sizes[i] = 0;
            }
            return;
        }
        let available = total - (visible - 1);
        if self.consistent(visible, available) {
            return;
        }
        let mut targets = vec![0; visible as usize];
        let mut mins = vec![0; visible as usize];
        let mut fixed = vec![false; visible as usize];
        for i in 0..visible as usize {
            mins[i] = self.min_for(i);
            fixed[i] = self.panes[i].fixed;
            if self.sizes[i] > 0 {
                targets[i] = self.sizes[i];
            } else {
                targets[i] = mins[i];
            }
        }
        let result = distribute_between(available, &targets, &mins, &fixed);
        for i in 0..visible as usize {
            self.sizes[i] = result[i];
        }
        for i in visible as usize..n {
            self.sizes[i] = 0;
        }
    }

    fn consistent(&self, visible: i32, available: i32) -> bool {
        let mut sum = 0;
        for i in 0..visible as usize {
            if self.sizes[i] < self.min_for(i) {
                return false;
            }
            sum += self.sizes[i];
        }
        for i in visible as usize..self.sizes.len() {
            if self.sizes[i] != 0 {
                return false;
            }
        }
        sum == available
    }

    fn correct_mins(&mut self, visible: i32) {
        let mut deficit = 0;
        for i in 0..visible as usize {
            let m = self.min_for(i);
            if self.sizes[i] < m {
                deficit += m - self.sizes[i];
                self.sizes[i] = m;
            }
        }
        if deficit == 0 {
            return;
        }
        let mut com_folga = Vec::new();
        for i in 0..visible as usize {
            if self.sizes[i] > self.min_for(i) {
                com_folga.push(i);
            }
        }
        self.reduce(&com_folga, deficit);
    }

    fn grow_focused(&mut self, focus: i32, delta: i32, visible: i32) {
        let focus = focus as usize;
        let mut donors = Vec::new();
        let mut folga = 0;
        for i in 0..visible as usize {
            if i == focus || self.panes[i].fixed {
                continue;
            }
            let avail = self.sizes[i] - self.min_for(i);
            if avail > 0 {
                donors.push(i);
                folga += avail;
            }
        }
        if folga == 0 {
            return;
        }
        let mover = delta.min(folga);
        self.reduce(&donors, mover);
        self.sizes[focus] += mover;
        self.dirty = true;
    }

    fn shrink_focused(&mut self, focus: i32, delta: i32, visible: i32) {
        let focus = focus as usize;
        let mut receptors = Vec::new();
        for i in 0..visible as usize {
            if i == focus || self.panes[i].fixed {
                continue;
            }
            receptors.push(i);
        }
        if receptors.is_empty() {
            for i in 0..visible as usize {
                if i != focus {
                    receptors.push(i);
                }
            }
        }
        if receptors.is_empty() {
            return;
        }
        let reduzir = delta.min(self.sizes[focus] - self.min_for(focus));
        if reduzir <= 0 {
            return;
        }
        self.sizes[focus] -= reduzir;
        self.increase(&receptors, reduzir);
        self.dirty = true;
    }

    fn reduce(&mut self, indices: &[usize], mut quantity: i32) {
        if quantity <= 0 || indices.is_empty() {
            return;
        }
        let mut folga_total = 0;
        for &i in indices {
            folga_total += self.sizes[i] - self.min_for(i);
        }
        if folga_total <= 0 {
            return;
        }
        quantity = quantity.min(folga_total);
        let mut used = 0;
        let mut restos = vec![0i64; indices.len()];
        for (j, &i) in indices.iter().enumerate() {
            let folga = (self.sizes[i] - self.min_for(i)) as i64;
            let numero = quantity as i64 * folga;
            let parte = (numero / folga_total as i64) as i32;
            self.sizes[i] -= parte;
            used += parte;
            restos[j] = numero % folga_total as i64;
        }
        let mut falta = quantity - used;
        for j in indices_with_largest_rest(&restos) {
            if falta == 0 {
                break;
            }
            self.sizes[indices[j]] -= 1;
            falta -= 1;
        }
    }

    fn increase(&mut self, indices: &[usize], quantity: i32) {
        if quantity <= 0 || indices.is_empty() {
            return;
        }
        let mut total = 0;
        for &i in indices {
            total += self.sizes[i];
        }
        if total <= 0 {
            let base = quantity / indices.len() as i32;
            let rem = quantity % indices.len() as i32;
            for (j, &i) in indices.iter().enumerate() {
                self.sizes[i] += base;
                if (j as i32) < rem {
                    self.sizes[i] += 1;
                }
            }
            return;
        }
        let mut used = 0;
        let mut restos = vec![0i64; indices.len()];
        for (j, &i) in indices.iter().enumerate() {
            let numero = quantity as i64 * self.sizes[i] as i64;
            let parte = (numero / total as i64) as i32;
            self.sizes[i] += parte;
            used += parte;
            restos[j] = numero % total as i64;
        }
        let mut falta = quantity - used;
        for j in indices_with_largest_rest(&restos) {
            if falta == 0 {
                break;
            }
            self.sizes[indices[j]] += 1;
            falta -= 1;
        }
    }

    fn handle_key(&mut self, key: &KeyMsg) {
        let tecla = key.to_string();
        match tecla.as_str() {
            "tab" => self.focus_next(),
            "shift+tab" => self.focus_prev(),
            "alt+left" => {
                if self.dir == Direction::Horizontal {
                    self.resize(-1);
                }
            }
            "alt+right" => {
                if self.dir == Direction::Horizontal {
                    self.resize(1);
                }
            }
            "alt+up" => {
                if self.dir == Direction::Vertical {
                    self.resize(-1);
                }
            }
            "alt+down" => {
                if self.dir == Direction::Vertical {
                    self.resize(1);
                }
            }
            "+" | "=" => self.resize(1),
            "-" | "_" => self.resize(-1),
            _ => {}
        }
    }

    fn render_horizontal(&self, visible: i32) -> String {
        let visible = visible as usize;
        let mut blocos: Vec<Vec<String>> = Vec::with_capacity(visible);
        for i in 0..visible {
            let ocultos = if i == visible - 1 {
                self.number_hidden(visible as i32)
            } else {
                0
            };
            let bloco = self.render_pane(i, self.sizes[i], self.height, ocultos);
            blocos.push(bloco.split('\n').map(|s| s.to_string()).collect());
        }
        let mut b = String::new();
        for linha in 0..self.height as usize {
            for i in 0..visible {
                if i > 0 {
                    b.push_str(DIVIDER_VERTICAL);
                }
                if linha < blocos[i].len() {
                    b.push_str(&blocos[i][linha]);
                }
            }
            if linha < self.height as usize - 1 {
                b.push('\n');
            }
        }
        b
    }

    fn render_vertical(&self, visible: i32) -> String {
        let visible = visible as usize;
        let mut b = String::new();
        for i in 0..visible {
            if i > 0 {
                b.push_str(&DIVIDER_HORIZONTAL.repeat(self.width as usize));
                b.push('\n');
            }
            let ocultos = if i == visible - 1 {
                self.number_hidden(visible as i32)
            } else {
                0
            };
            b.push_str(&self.render_pane(i, self.width, self.sizes[i], ocultos));
            if i < visible - 1 {
                b.push('\n');
            }
        }
        b
    }

    fn render_pane(&self, i: usize, largura: i32, altura: i32, ocultos: i32) -> String {
        if largura <= 0 || altura <= 0 {
            return String::new();
        }
        if altura == 1 {
            return render_content(&self.panes[i].content, largura, 1);
        }
        let titulo = line_title(
            &self.panes[i].title,
            i as i32 == self.focus,
            largura,
            ocultos,
        );
        let corpo = render_content(&self.panes[i].content, largura, altura - 1);
        format!("{}\n{}", titulo, corpo)
    }
}

impl charmed_bubbletea::Model for Layout {
    fn init(&self) -> Option<Cmd> {
        self.init()
    }
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        self.update(msg)
    }
    fn view(&self) -> String {
        self.view()
    }
}

fn normalize_min(m: i32) -> i32 {
    if m <= 0 {
        DEFAULT_MIN
    } else {
        m
    }
}

fn indices_with_largest_rest(restos: &[i64]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..restos.len()).collect();
    order.sort_by(|&a, &b| restos[b].cmp(&restos[a]).then_with(|| a.cmp(&b)));
    order
}

fn distribute_between(total: i32, targets: &[i32], mins: &[i32], fixed: &[bool]) -> Vec<i32> {
    let n = mins.len();
    let mut result = vec![0; n];
    let mut fixed_idx = Vec::new();
    let mut flex_idx = Vec::new();
    let mut soma_min_fixed = 0;
    let mut soma_min_flex = 0;
    let mut soma_target_fixed = 0;
    for i in 0..n {
        if fixed[i] {
            fixed_idx.push(i);
            soma_min_fixed += mins[i];
            soma_target_fixed += targets[i].max(mins[i]);
        } else {
            flex_idx.push(i);
            soma_min_flex += mins[i];
        }
    }
    let mut orcamento_fixos = soma_target_fixed;
    let maximo = total - soma_min_flex;
    if orcamento_fixos > maximo {
        orcamento_fixos = maximo;
    }
    orcamento_fixos = orcamento_fixos.max(soma_min_fixed);

    let mut alvos_fixos = Vec::new();
    let mut mins_fixos = Vec::new();
    for &i in &fixed_idx {
        alvos_fixos.push(targets[i]);
        mins_fixos.push(mins[i]);
    }
    let fatias_fixas = divide_with_mins(orcamento_fixos, &alvos_fixos, &mins_fixos);
    for (j, &i) in fixed_idx.iter().enumerate() {
        result[i] = fatias_fixas[j];
    }

    let mut alvos_flex = Vec::new();
    let mut mins_flex = Vec::new();
    for &i in &flex_idx {
        alvos_flex.push(targets[i]);
        mins_flex.push(mins[i]);
    }
    let fatias_flex = divide_with_mins(total - orcamento_fixos, &alvos_flex, &mins_flex);
    for (j, &i) in flex_idx.iter().enumerate() {
        result[i] = fatias_flex[j];
    }
    result
}

fn divide_with_mins(budget: i32, targets: &[i32], mins: &[i32]) -> Vec<i32> {
    let n = mins.len();
    let mut slices = vec![0; n];
    if n == 0 {
        return slices;
    }
    let mut soma_min = 0;
    for i in 0..n {
        slices[i] = mins[i];
        soma_min += mins[i];
    }
    let sobra = budget - soma_min;
    if sobra <= 0 {
        return slices;
    }
    let mut weights = vec![0; n];
    let mut soma_pesos = 0;
    for i in 0..n {
        let mut peso = targets[i] - mins[i];
        if peso < 0 {
            peso = 0;
        }
        weights[i] = peso;
        soma_pesos += peso;
    }
    if soma_pesos == 0 {
        let base = sobra / n as i32;
        let rem = sobra % n as i32;
        for i in 0..n {
            slices[i] += base;
            if (i as i32) < rem {
                slices[i] += 1;
            }
        }
        return slices;
    }
    let mut used = 0;
    let mut restos = vec![0i64; n];
    for i in 0..n {
        let numero = sobra as i64 * weights[i] as i64;
        let parte = (numero / soma_pesos as i64) as i32;
        slices[i] += parte;
        used += parte;
        restos[i] = numero % soma_pesos as i64;
    }
    let mut falta = sobra - used;
    for i in indices_with_largest_rest(&restos) {
        if falta == 0 {
            break;
        }
        slices[i] += 1;
        falta -= 1;
    }
    slices
}

fn render_content(content: &str, largura: i32, altura: i32) -> String {
    if largura <= 0 || altura <= 0 {
        return String::new();
    }
    let altura_usize = altura as usize;
    let style = Style::new().width(largura).height(altura);
    let bloco = style.render(content);
    let mut linhas: Vec<String> = bloco.split('\n').map(|s| s.to_string()).collect();
    if linhas.len() > altura_usize {
        linhas.truncate(altura_usize);
    }
    while linhas.len() < altura_usize {
        linhas.push(String::new());
    }
    for line in &mut linhas {
        *line = pad_to(line, largura);
    }
    linhas.join("\n")
}

fn line_title(titulo: &str, focado: bool, largura: i32, ocultos: i32) -> String {
    if largura <= 0 {
        return String::new();
    }
    let largura = largura as usize;
    let marcador = if focado {
        MARKER_FOCUSED
    } else {
        MARKER_UNFOCUSED
    };
    let mut sufixo = String::new();
    if ocultos > 0 {
        sufixo = hidden_label(ocultos);
        if !titulo.is_empty() {
            sufixo = format!(" {}", sufixo);
        }
    }
    let largura_marcador = charm_core::display_width(marcador);
    if largura <= largura_marcador {
        return pad_to(&ansi_truncate(marcador, largura), largura as i32);
    }
    let largura_sufixo = charm_core::display_width(&sufixo);
    if largura_sufixo > 0 && largura_marcador + largura_sufixo > largura {
        return pad_to(&sufixo, largura as i32);
    }
    let disponivel =
        (largura as i32 - largura_marcador as i32 - largura_sufixo as i32).max(0) as usize;
    let texto_titulo = ansi_truncate(titulo, disponivel);
    pad_to(
        &format!("{}{}{}", marcador, texto_titulo, sufixo),
        largura as i32,
    )
}

fn hidden_label(n: i32) -> String {
    if n <= 0 {
        String::new()
    } else {
        format!("+{}", n)
    }
}

fn pad_to(linha: &str, largura: i32) -> String {
    let largura = largura.max(0) as usize;
    if largura == 0 {
        return String::new();
    }
    let w = charm_core::display_width(linha);
    if w > largura {
        return ansi_truncate(linha, largura);
    }
    if w < largura {
        return format!("{}{}", linha, " ".repeat(largura - w));
    }
    linha.to_string()
}

fn ansi_truncate(s: &str, max_width: usize) -> String {
    if charm_core::display_width(s) <= max_width {
        return s.to_string();
    }
    let mut out = String::new();
    let mut width = 0usize;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            out.push(c);
            if let Some(&'[') = chars.peek() {
                out.push(chars.next().unwrap());
                while let Some(&nc) = chars.peek() {
                    out.push(chars.next().unwrap());
                    let code = nc as u32;
                    if (0x40..=0x7E).contains(&code) {
                        break;
                    }
                }
            } else if let Some(&']') = chars.peek() {
                out.push(chars.next().unwrap());
                while let Some(nc) = chars.next() {
                    out.push(nc);
                    if nc == '\x07' {
                        break;
                    }
                    if nc == '\x1b' {
                        if let Some(&'\\') = chars.peek() {
                            out.push(chars.next().unwrap());
                            break;
                        }
                    }
                }
            } else if chars.peek().is_some() {
                out.push(chars.next().unwrap());
            }
            continue;
        }
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if width + w > max_width {
            break;
        }
        out.push(c);
        width += w;
    }
    out
}
