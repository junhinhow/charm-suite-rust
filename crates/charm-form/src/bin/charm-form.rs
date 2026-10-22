#![forbid(unsafe_code)]

//! Command-line runner for YAML-defined forms.

use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use charm_form::{load_from_file, renderer, Answers, FormSpec};

#[derive(Debug, PartialEq, Eq)]
struct Config {
    file: Option<PathBuf>,
    output: Option<PathBuf>,
    json: bool,
    help: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            file: None,
            output: None,
            json: true,
            help: false,
        }
    }
}

fn main() {
    if let Err(error) = run(std::env::args().skip(1)) {
        eprintln!("erro: {error}");
        std::process::exit(1);
    }
}

fn run<I>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = String>,
{
    let config = parse_args(args)?;
    if config.help {
        print_help();
        return Ok(());
    }
    let Some(path) = config.file else {
        print_help();
        return Err("informe o arquivo do formulário com -file".to_owned());
    };
    let spec = load_from_file(&path).map_err(|error| error.to_string())?;
    let answers = renderer::run(spec.clone()).map_err(|error| error.to_string())?;
    if let Some(path) = config.output {
        let mut file = File::create(&path).map_err(|error| {
            format!("erro ao criar arquivo de saída {}: {error}", path.display())
        })?;
        write_result(&mut file, &spec, &answers, config.json)?;
    } else {
        let stdout = io::stdout();
        let mut output = stdout.lock();
        write_result(&mut output, &spec, &answers, config.json)?;
    }
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Config, String>
where
    I: IntoIterator<Item = String>,
{
    let mut config = Config::default();
    let mut values = args.into_iter();
    while let Some(argument) = values.next() {
        let (name, inline_value) = argument
            .split_once('=')
            .map_or((argument.as_str(), None), |(name, value)| {
                (name, Some(value))
            });
        match name {
            "-h" | "--help" => config.help = true,
            "-file" | "--file" => {
                let value = take_value(name, inline_value, &mut values)?;
                config.file = Some(PathBuf::from(value));
            }
            "-out" | "--out" => {
                let value = take_value(name, inline_value, &mut values)?;
                config.output = Some(PathBuf::from(value));
            }
            "-json" | "--json" => {
                config.json = match inline_value {
                    Some("true") | None => true,
                    Some("false") => false,
                    Some(value) => return Err(format!("valor inválido para {name}: {value}")),
                };
            }
            _ if argument.starts_with('-') => {
                return Err(format!("opção desconhecida: {argument}"));
            }
            _ if config.file.is_none() => config.file = Some(PathBuf::from(argument)),
            _ => return Err(format!("argumento inesperado: {argument}")),
        }
    }
    Ok(config)
}

fn take_value<I>(name: &str, inline_value: Option<&str>, values: &mut I) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    inline_value
        .map(str::to_owned)
        .or_else(|| values.next())
        .ok_or_else(|| format!("faltou o valor para {name}"))
}

fn print_help() {
    let _ = writeln!(
        io::stderr().lock(),
        "Uso: charm-form [opções] [arquivo.yaml]\n\nExecuta o formulário definido em YAML e emite as respostas.\n\n  -file CAMINHO  arquivo YAML do formulário\n  -out CAMINHO   arquivo de saída (padrão: stdout)\n  -json=false    imprime um resumo legível em vez de JSON\n  -h, --help     mostra esta ajuda"
    );
}

fn write_result<W: Write>(
    writer: &mut W,
    spec: &FormSpec,
    answers: &Answers,
    json: bool,
) -> Result<(), String> {
    if json {
        answers
            .write_json(writer)
            .map_err(|error| error.to_string())
    } else {
        writer
            .write_all(summary(spec, answers).as_bytes())
            .map_err(|error| format!("erro ao escrever resumo: {error}"))
    }
}

fn summary(spec: &FormSpec, answers: &Answers) -> String {
    let mut output = String::new();
    for field in &spec.fields {
        if let Some(value) = answers.get(&field.name) {
            output.push_str(&field.name);
            output.push_str(": ");
            output.push_str(&format_value(value));
            output.push('\n');
        }
    }
    output
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Bool(true) => "sim".to_owned(),
        serde_json::Value::Bool(false) => "não".to_owned(),
        serde_json::Value::Array(values) => values
            .iter()
            .map(format_value)
            .collect::<Vec<_>>()
            .join(", "),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_go_style_cli_options_and_positional_path() {
        let config = parse_args([
            "-out=result.json".to_owned(),
            "-json=false".to_owned(),
            "form.yaml".to_owned(),
        ])
        .expect("args should parse");
        assert_eq!(config.file, Some(PathBuf::from("form.yaml")));
        assert_eq!(config.output, Some(PathBuf::from("result.json")));
        assert!(!config.json);
    }

    #[test]
    fn reports_missing_form_file() {
        assert!(run(Vec::<String>::new()).is_err());
    }

    #[test]
    fn renders_summary_in_spec_order() {
        let spec = FormSpec {
            title: "T".to_owned(),
            fields: vec![
                charm_form::FieldSpec {
                    name: "name".to_owned(),
                    ..Default::default()
                },
                charm_form::FieldSpec {
                    name: "active".to_owned(),
                    ..Default::default()
                },
                charm_form::FieldSpec {
                    name: "tags".to_owned(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let answers = [
            ("name".to_owned(), serde_json::json!("Ana")),
            ("active".to_owned(), serde_json::json!(true)),
            ("tags".to_owned(), serde_json::json!(["a", "b"])),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            summary(&spec, &answers),
            "name: Ana\nactive: sim\ntags: a, b\n"
        );
    }

    #[test]
    fn formats_supported_summary_values() {
        assert_eq!(format_value(&serde_json::Value::Null), "");
        assert_eq!(format_value(&serde_json::json!("text")), "text");
        assert_eq!(format_value(&serde_json::json!(false)), "não");
        assert_eq!(format_value(&serde_json::json!(42)), "42");
    }
}
