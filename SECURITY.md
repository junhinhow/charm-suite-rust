# PolÃ­tica de seguranÃ§a

## VersÃµes suportadas

| VersÃ£o | Suporte de seguranÃ§a |
|---|---|
| 0.1.x | âœ… Sim |
| < 0.1 | âŒ NÃ£o (nÃ£o hÃ¡ versÃµes anteriores publicadas) |

Quando uma nova versÃ£o menor for publicada (ex.: 0.2.0), a sÃ©rie 0.1.x deixa de
receber correÃ§Ãµes, salvo se houver anÃºncio explÃ­cito em contrÃ¡rio.

## Como reportar uma vulnerabilidade

**NÃ£o abra issue pÃºblica** (nem no Forgejo, nem no GitHub) para vulnerabilidades.

Envie um e-mail para o mantenedor:

- **E-mail:** `aaltairjr@gmail.com`

Inclua, se possÃ­vel:

- descriÃ§Ã£o do problema e o impacto esperado;
- crate e versÃ£o afetados;
- passos para reproduzir ou um caso de teste mÃ­nimo;
- sugestÃ£o de correÃ§Ã£o, se houver.

Use o assunto `[SEGURANÃ‡A] charm-<crate>: <resumo>`.

## Prazos de resposta

| Etapa | Prazo |
|---|---|
| ConfirmaÃ§Ã£o do recebimento | atÃ© 72 horas |
| Triagem e classificaÃ§Ã£o de severidade | atÃ© 7 dias |
| CorreÃ§Ã£o ou plano de mitigaÃ§Ã£o | atÃ© 30 dias para severidade alta ou crÃ­tica |
| PublicaÃ§Ã£o do aviso | apÃ³s a versÃ£o corrigida estar no crates.io |

Os prazos sÃ£o metas, nÃ£o garantias. Mantenedor Ãºnico: prazos podem variar.

## DivulgaÃ§Ã£o coordenada

Pedimos que a vulnerabilidade nÃ£o seja divulgada publicamente antes da correÃ§Ã£o
estar disponÃ­vel. Quando o reporte for aceito, o crÃ©dito ao pesquisador Ã© dado no
aviso, se ele desejar.

## Escopo

Vulnerabilidades em qualquer crate `charm-*` deste workspace sÃ£o bem-vindas.
Pontos com maior atenÃ§Ã£o:

- **`charm-notify`**: monta comandos de processos externos (`notify-send`,
  `osascript`, `powershell`, `msg`) a partir de texto do usuÃ¡rio. Escapes e
  argumentos sÃ£o crÃ­ticos.
- **`charm-form`** e **`charm-i18n`**: leem arquivos YAML e JSON fornecidos pelo usuÃ¡rio.
- **`charm-table-cli`**: lÃª entradas arbitrÃ¡rias (CSV, TSV, JSON, JSON Lines) pela linha de comando.

Problemas em dependÃªncias de terceiros devem ser reportados aos respectivos
projetos. Se o impacto recair sobre este workspace, avise-nos tambÃ©m.
