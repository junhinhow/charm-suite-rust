//! Demonstrate deriving a component tree from explicitly marked fields.

use charm_devtools::{ComponentNode, DevInspect};

#[derive(DevInspect)]
struct Button;

#[derive(DevInspect)]
struct Toolbar {
    #[inspect]
    save: Button,
    #[devtools(inspect)]
    actions: Vec<Button>,
    #[inspect(skip)]
    cache: Button,
}

fn main() {
    let toolbar = Toolbar {
        save: Button,
        actions: vec![Button, Button],
        cache: Button,
    };
    let _ = &toolbar.cache;
    let tree: ComponentNode = toolbar.inspect();
    println!("{}", tree.name);
    for child in tree.children {
        println!("{}: {}", child.name, child.type_name);
    }
}
