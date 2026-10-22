//! CLI do harness de paridade.
//!
//! Uso:
//!
//! ```text
//! # Gera um golden file executando um comando (tipicamente o Go original)
//! cargo run -p charm-parity --bin parity -- gen crates/charm-nav/tests/golden/tabs.txt -- go run ./reference/go/charm-nav/cmd/tabs.go
//!
//! # Compara a saida de um comando com um golden file
//! cargo run -p charm-parity --bin parity -- check crates/charm-nav/tests/golden/tabs.txt -- cargo run -p charm-nav --example tabs
//!
//! # Normaliza uma saida (remove ANSI) e imprime
//! cargo run -p charm-parity --bin parity -- normalize -- cargo run -p charm-nav --example tabs
//! ```

use std::io::Write as _;
use std::process::Command;

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("gen") => cmd_gen(&args[1..]),
        Some("check") => cmd_check(&args[1..]),
        Some("normalize") => cmd_normalize(&args[1..]),
        Some("help" | "--help" | "-h") => {
            print_help();
            Ok(())
        }
        _ => {
            print_help();
            Err("informe um subcomando: gen | check | normalize".to_owned())
        }
    };

    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!(
        "harness de paridade Charm Suite

uso:
  parity gen <arquivo-golden> -- <comando> [args...]   gera o golden file
  parity check <arquivo-golden> -- <comando> [args...]  compara comando x golden
  parity normalize -- <comando> [args...]               imprime saida sem ANSI"
    );
}

/// Extrai o comando (tudo apos `--`) dos argumentos.
fn split_command(args: &[String]) -> Result<Vec<String>, String> {
    let pos = args
        .iter()
        .position(|a| a == "--")
        .ok_or_else(|| "faltou o separador `--` antes do comando".to_owned())?;
    let cmd: Vec<String> = args[pos + 1..].to_vec();
    if cmd.is_empty() {
        return Err("nenhum comando informado".to_owned());
    }
    Ok(cmd)
}

fn run_capture(cmd: &[String]) -> Result<String, String> {
    let out = Command::new(&cmd[0])
        .args(&cmd[1..])
        .output()
        .map_err(|e| format!("falha ao executar `{}`: {e}", cmd[0]))?;
    if !out.status.success() {
        return Err(format!(
            "comando `{}` falhou com status {}\n{}",
            cmd[0],
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn cmd_gen(args: &[String]) -> Result<(), String> {
    let sep = args
        .iter()
        .position(|a| a == "--")
        .ok_or_else(|| "uso: parity gen <arquivo-golden> -- <comando>".to_owned())?;
    let target = args[..sep].join(" ");
    if target.is_empty() {
        return Err("informe o caminho do golden file".to_owned());
    }
    let command = split_command(args)?;
    let output = run_capture(&command)?;
    if let Some(parent) = std::path::Path::new(&target).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut f = std::fs::File::create(&target).map_err(|e| format!("{e}"))?;
    f.write_all(output.as_bytes()).map_err(|e| format!("{e}"))?;
    println!("golden gerado: {target} ({} bytes)", output.len());
    Ok(())
}

fn cmd_check(args: &[String]) -> Result<(), String> {
    let sep = args
        .iter()
        .position(|a| a == "--")
        .ok_or_else(|| "uso: parity check <arquivo-golden> -- <comando>".to_owned())?;
    let target = args[..sep].join(" ");
    let command = split_command(args)?;
    let output = run_capture(&command)?;
    let expected = std::fs::read_to_string(&target)
        .map_err(|e| format!("nao foi possivel ler `{target}`: {e}"))?;
    let e = charm_parity::normalize(&expected);
    let a = charm_parity::normalize(&output);
    if e == a {
        println!("OK: `{target}` confere");
        Ok(())
    } else {
        Err(format!(
            "divergencia em `{target}`:\n--- esperado ---\n{e}\n--- obtido ---\n{a}\n--- fim ---"
        ))
    }
}

fn cmd_normalize(args: &[String]) -> Result<(), String> {
    let command = split_command(args)?;
    let output = run_capture(&command)?;
    println!("{}", charm_parity::normalize(&output));
    Ok(())
}
