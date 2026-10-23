# charm-input

Date, time, range, and masked-text inputs for Bubble Tea.

> [Leia em Português](README.pt-br.md)

## Install

```bash
cargo add charm-input
```

## Usage

```rust
use charm_input::Date;
use chrono::NaiveDate;

let mut date = Date::new(NaiveDate::from_ymd_opt(2026, 10, 22).expect("valid"));
date.set_size(40, 8);
date.focus();
```

Each component implements `charmed_bubbletea::Model`.

## API

See rustdoc for the full API: `Date`, `Time`, `Range`, `Masked`, `DateSelectedMsg`.

## Known differences

- Components do not call `time::now`; today highlighting comes from `set_today`.
- `NO_COLOR` switches the range bar and masked cursor to ASCII.

## License

MIT
