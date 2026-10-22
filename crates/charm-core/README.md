# charm-core

Primitivos compartilhados da **Charm Suite Rust**.

Este crate e de uso **interno**. Ele centraliza o tipo de cor canonico
[`Rgb`], utilitarios de texto ([`strip_ansi`], [`display_width`],
[`pad_or_truncate`], [`split_lines`], [`truncate`]) e o erro padrao [`Error`].

A API publica e instavel — apenas os crates da suite devem depender dela
diretamente. Usuarios finais devem usar as bibliotecas individuais
(`charm-color`, `charm-layout`, etc.).

## Licenca

MIT
