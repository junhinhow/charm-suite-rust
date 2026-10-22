//! Integration tests for the optional derive macro.

#![cfg(feature = "derive")]

use charm_devtools::DevInspect;

struct Leaf;

impl DevInspect for Leaf {}

struct Opaque;

#[derive(DevInspect)]
struct Container<T> {
    #[inspect]
    item: T,
    #[devtools(inspect)]
    items: Vec<Leaf>,
    #[inspect(skip)]
    skipped: Opaque,
    ignored: Opaque,
}

#[derive(DevInspect)]
struct Tuple(Leaf);

#[derive(DevInspect)]
struct Unit;

#[test]
fn derive_inspects_marked_fields_and_expands_vectors() {
    let container = Container {
        item: Leaf,
        items: vec![Leaf, Leaf],
        skipped: Opaque,
        ignored: Opaque,
    };
    let _ = (&container.skipped, &container.ignored);
    let node = container.inspect();

    assert_eq!(node.name, "Container");
    assert!(node.type_name.ends_with("Container<derive::Leaf>"));
    assert_eq!(
        node.children
            .iter()
            .map(|child| child.name.as_str())
            .collect::<Vec<_>>(),
        ["item", "items[0]", "items[1]"]
    );
    assert!(node
        .children
        .iter()
        .all(|child| child.type_name.ends_with("Leaf")));
}

#[test]
fn tuple_and_unit_structs_only_produce_a_root() {
    let tuple = Tuple(Leaf).inspect();
    let unit = Unit.inspect();
    assert_eq!(tuple.name, "Tuple");
    assert_eq!(tuple.children, Vec::new());
    assert_eq!(unit.name, "Unit");
    assert_eq!(unit.children, Vec::new());
}
