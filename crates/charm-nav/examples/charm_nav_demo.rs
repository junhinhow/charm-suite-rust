//! Demo of the `charm-nav` crate: tabs, a breadcrumb and a sidebar.
//!
//! Run it with:
//!
//! ```text
//! cargo run -p charm-nav --example demo
//! ```

use charm_nav::{Breadcrumb, Sidebar, Tabs};

fn main() {
    println!("== Tabs ==");
    let mut tabs = Tabs::new(&["Home", "Settings", "About"]);
    println!("{}", tabs.render());
    tabs.next();
    println!("{}", tabs.render());
    tabs.previous();
    tabs.previous();
    println!("{} (wrapped around)", tabs.render());

    println!();
    println!("== Breadcrumb ==");
    let mut breadcrumb = Breadcrumb::new(&["Home", "Settings"]);
    breadcrumb.push("Profile");
    println!("{}", breadcrumb.render());
    println!("popped: {}", breadcrumb.pop());
    println!("{}", breadcrumb.render());

    println!();
    println!("== Sidebar ==");
    let mut sidebar = Sidebar::new(&["Dashboard", "Users", "Settings", "Logs"]);
    sidebar.set_selected(1);
    println!("{}", sidebar.render());
    sidebar.next();
    println!("{}", sidebar.render());
}
