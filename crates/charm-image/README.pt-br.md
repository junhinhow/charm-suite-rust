# charm-image

Renderize imagens no terminal usando Kitty, iTerm2, Sixel, half-blocks ANSI truecolor ou um fallback monocromático ASCII.

> **[Read in English](README.md)**

Este é o porte Rust da referência Go local em `reference/go/charm-image`. As imagens são fornecidas como [`image::DynamicImage`].

## Instalação

```toml
[dependencies]
charm-image = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
```

Habilite na sua dependência `image` os formatos necessários. `charm-image` ativa codificação PNG para Kitty e iTerm2.

## Uso

```rust,no_run
use charm_image::{render, Options};
use image::ImageReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let imagem = ImageReader::open("foto.png")?.decode()?;
    let saida = render(&imagem, Options::default().width(60))?;
    print!("{saida}");
    Ok(())
}
```

Force um protocolo ou use diretamente o renderizador ASCII:

```rust
use charm_image::{ascii_art, render, Options, Protocol};
# use image::{DynamicImage, Rgba, RgbaImage};
# let imagem = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255])));
let saida = render(&imagem, Options::default()
    .protocol(Protocol::HalfBlock)
    .width(40)
    .height(20))?;
let monocromatica = ascii_art(&imagem, 40, 0, "");
# let _ = (saida, monocromatica);
# Ok::<(), charm_image::Error>(())
```

## Detecção de protocolo

| Indicador do terminal | Protocolo |
| --- | --- |
| `TERM=dumb` ou `NO_COLOR` não vazio | ASCII |
| `KITTY_WINDOW_ID` | Kitty |
| `TERM_PROGRAM=iTerm.app`, `WezTerm` ou `mintty` | iTerm2 |
| `TERM` ou `COLORTERM` contendo `sixel` | Sixel |
| Caso contrário | Half-block ANSI truecolor |

A detecção usa variáveis de ambiente e não consulta o terminal. `Options::protocol` explícito tem precedência.

## API

| Símbolo | Descrição |
| --- | --- |
| `render`, `Options` | Entrada principal e opções de dimensões/protocolo/dithering/fundo |
| `Protocol`, `detect`, `detect_from` | Protocolos e detecção pelo ambiente |
| `ascii_art` | Renderizador monocromático com rampa de caracteres configurável |
| `render_half_block` | Renderizador universal `▀` com truecolor em foreground/background |
| `render_kitty`, `render_iterm2`, `render_sixel` | Sequências de escape cruas dos protocolos |
| `Error` | Erros de imagem vazia, protocolo inválido, tamanho e codificação PNG |

## Comportamento e limitações

- A largura natural é a largura original limitada a 80 células; a altura ausente preserva aproximadamente a proporção do terminal (células consideradas duas vezes mais altas do que largas).
- Half-block, ASCII e Sixel compõem pixels transparentes sobre `Options::background`, ou preto por padrão. Kitty/iTerm2 preservam o alfa do PNG.
- O dithering ASCII aplica um deslocamento ordenado Bayer 4×4 à luminância; não afeta a saída colorida.
- Kitty envia PNG/base64 em sequências APC de até 4096 bytes. iTerm2 usa OSC 1337.
- Sixel usa raster de 8×16 pixels por célula, cubo de 216 cores mais 40 cinzas e run-length encoding. Como na referência Go, o payload Sixel é codificado em base64 dentro do DCS; terminais convencionais esperam Sixel cru, portanto decodifique o payload antes de enviá-lo diretamente ao terminal.
- `render` posiciona o cursor na linha 1, coluna 1 antes da saída Sixel; `render_sixel` retorna somente a sequência DCS bruta.
- Os protocolos são cobertos por testes unitários, não por integração com terminais reais. Half-block requer truecolor; o redimensionamento usa média simples sem correção de gamma nem rotação.
- A API Rust recebe `&DynamicImage`, portanto imagem nil é impossível de representar; imagens sem dimensões retornam `Error::EmptyImage`.

## Licença

MIT
