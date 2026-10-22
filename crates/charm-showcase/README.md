# charm-showcase

An offline, deterministic terminal tour through the Charm Suite Rust crates.
It combines the Dracula theme, banner, navigation and flex layout, charts,
three spinner frames, a unified diff, confirmation dialog, toast, generated
color palette, English/Portuguese translations, an executed three-step wizard,
and a YAML form embedded in the binary and printed as JSON.

Run it with:

```sh
cargo run -p charm-showcase
```

There is no interactive loop, sleep, timestamp, or random input. ANSI styling is
removed before printing so the output is stable in terminals and CI logs.

## Text screenshot (selected output)

```text
=== CharmSuite | Unified Demo ===
░▄███░ █░░░█░ ░▄██▄░ ████▄░ █▀░▀█░ ░░░░░░ ░▄███░ █░░░█░ █████░ █████░ █████░
█▀░░░░ █░░░█░ █▀░░█▀ █░░░█░ ██░██░ ░░░░░░ █▀░░░░ █░░░█░ ░░█░░░ ░░█░░░ █░░░░░
█░░░░░ █████░ █████░ ████▀░ █░█░█░ ░░░░░░ ░▀██▄░ █░░░█░ ░░█░░░ ░░█░░░ ████░░
█▄░░░░ █░░░█░ █░░░█░ █░░█░░ █░░░█░ ░░░░░░ ░░░░█░ █▄░▄█░ ░░█░░░ ░░█░░░ █░░░░░
░▀███░ █░░░█░ ▀░░░▀░ █░░░█░ █░░░█░ ░░░░░░ ████▀░ ░▀█▀░░ █████░ ░░█░░░ █████░
CharmSuite Showcase
Theme: dracula (#bd93f9)

[Navigation + Flex Row]
 ▸ Dashboard    Overview | Components | Form
     Charts     All systems ready
     Settings

[Charts]
Sparkline:  ▃▁▆▄█
Progress: Build ████████████░░░░░░  72%

[Spinner: three deterministic frames]
⠋ ⠙ ⠹

[Diff]
@@ -1,2 +1,2 @@
   1      -theme = "nord"
   2      -status = "idle"
        1 +theme = "dracula"
        2 +status = "ready"

[Confirmation Dialog]
╭───────────────────────────────────╮
│            Deploy demo?           │
│  Apply the sample configuration?  │
│            Yes      No            │
╰───────────────────────────────────╯
[Toast]
╭──────────────────────────╮
│ [v]  Configuration saved │
╰──────────────────────────╯

[Color Palette]
 #bd93f9  #f9939c  #cff993  #93f9f0

[Translations]
en=Dashboard | pt-BR=Painel

[Executed Wizard]
╭──────────────────╮
│ Three-step setup │
╰──────────────────╯
Step 1 of 3: Select the Dracula theme
...
Step 2 of 3: Prepare the dashboard
...
Step 3 of 3: Mark the demo ready
...
✓ Three-step setup completed successfully!

[Embedded YAML Form -> JSON]
{
  "title": "Theme setup",
  "description": "Choose a theme for the sample terminal application.",
  "fields": [ ... ]
}
```

## Português (Brasil)

Uma demonstração offline e determinística das bibliotecas Rust da Charm Suite.
Ela compõe o tema Dracula, banner, navegação e layout flex, gráficos, três
quadros de spinner, diff, diálogo de confirmação, toast, paleta de cores,
traduções em inglês e português, wizard de três etapas executado e um formulário
YAML embutido convertido para JSON.

Execute com `cargo run -p charm-showcase`. Não há loop interativo, espera,
timestamp ou entrada aleatória. Os códigos ANSI são removidos antes da saída,
que permanece estável em terminais e no CI. A captura textual acima representa a
saída do programa.
