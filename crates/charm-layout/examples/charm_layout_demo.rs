//! Demo of the `charm-layout` crate: a flex row, a flex column and a grid.
//!
//! Run it with:
//!
//! ```text
//! cargo run -p charm-layout --example demo
//! ```

use charm_layout::{Direction, Flex, Grid};

fn main() {
    println!("== Flex row ==");
    let row = Flex::new(Direction::Row)
        .set_size(60, 5)
        .set_gap(2)
        .add_child(0.3, "Sidebar")
        .add_child(0.7, "Main content");
    println!("{}", row.render());

    println!();
    println!("== Flex column ==");
    let column = Flex::new(Direction::Column)
        .set_size(40, 12)
        .set_gap(1)
        .add_child(0.2, "Header")
        .add_child(0.6, "Body")
        .add_child(0.2, "Footer");
    println!("{}", column.render());

    println!();
    println!("== Grid ==");
    let grid = Grid::new(3)
        .set_size(60, 9)
        .set_gap(1)
        .add_children(["Cell 1", "Cell 2", "Cell 3", "Cell 4", "Cell 5"]);
    println!("{}", grid.render());
}
