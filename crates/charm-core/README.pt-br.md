# charm-core

Shared primitives of the **Charm Suite Rust**.

This crate is for **internal** use. It centralises the canonical colour type
[`Rgb`], text utilities ([`strip_ansi`], [`display_width`], [`pad_or_truncate`],
[`split_lines`], [`truncate`]) and the standard [`Error`].

The public API is unstable — only the suite's own crates should depend on it
directly. End users should depend on the individual libraries (`charm-color`,
`charm-layout`, …).

## License

MIT
