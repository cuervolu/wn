mod updater;

use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use miette::IntoDiagnostic;
use owo_colors::OwoColorize;
use rustyline::{DefaultEditor, error::ReadlineError};
use wn_diagnostics::{SourceFile, WnDiagnostic};
use wn_vm::{
    compiler::{compilar, compilar_repl},
    vm::VM,
};

use crate::updater::run_update;
use wn::{lexer::{Lexer, tokenizar}, parser::parsear};

#[derive(Parser)]
#[command(
    name = "wn",
    about = "El intérprete del lenguaje WN++",
    version,
    long_version = concat!(
    env!("CARGO_PKG_VERSION"), "\n",
    "commit: ", env!("GIT_HASH", "desconocido"),
    )
)]
struct Cli {
    /// Archivo fuente a ejecutar (.cl)
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Busca e instala la última versión de WN++
    Update {
        /// Fuerza la actualización aunque ya tengas la última versión
        #[arg(long)]
        force: bool,
    },
    /// Desinstala wn++ del sistema para siempre
    Uninstall,
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Update { force }) => run_update(force),
        Some(Command::Uninstall) => run_uninstall(),
        None => match cli.file {
            Some(path) => run_file(path),
            None => {
                run_repl();
                Ok(())
            }
        },
    }
}

fn run_file(path: PathBuf) -> miette::Result<()> {
    if path.extension().and_then(|e| e.to_str()) != Some("cl") {
        miette::bail!("se esperaba un archivo .cl, encontré '{}'", path.display());
    }

    let src = std::fs::read_to_string(&path).into_diagnostic()?;
    let filename = path.to_string_lossy();
    let source = Arc::new(SourceFile::new(filename.to_string(), src.clone()));

    let tokens = Lexer::new(&src).with_filename(&*filename).tokenizar()?;
    let stmts = parsear(tokens, &src, &filename)?;
    let chunk = compilar(&stmts, source)?;

    let mut vm = VM::new();
    let _ = vm.run(&chunk)?;

    Ok(())
}

fn run_repl() {
    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("{} {e}", "error:".red().bold());
            return;
        }
    };

    let _ = rl.load_history(".wn_history");

    let mut vm = VM::new();

    // Banner de bienvenida
    println!(
        "{} {} — escribe {} para salir",
        "WN++".cyan().bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed(),
        "'chao'".yellow(),
    );

    let prompt = format!("{} ", ">>>".cyan().bold());

    loop {
        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if matches!(trimmed, "chao" | "exit" | "quit") {
                    println!("{}", "¡Chao!".dimmed());
                    break;
                }

                let _ = rl.add_history_entry(trimmed);
                let source = Arc::new(SourceFile::new("<repl>", trimmed));

                let result = tokenizar(trimmed)
                    .and_then(|tokens| parsear(tokens, trimmed, "<repl>"))
                    .and_then(|stmts| compilar_repl(&stmts, source))
                    .and_then(|chunk| vm.run(&chunk));

                match result {
                    Ok(wn_vm::value::Value::Nada) => {}
                    Ok(val) => println!("{val}"),
                    Err(e) => eprint_error(e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "(usa 'chao' para salir)".dimmed());
            }
            Err(ReadlineError::Eof) => {
                println!("\n{}", "¡Chao!".dimmed());
                break;
            }
            Err(e) => {
                eprintln!("{} {e}", "error del REPL:".red().bold());
                break;
            }
        }
    }

    let _ = rl.save_history(".wn_history");
}

fn run_uninstall() -> miette::Result<()> {
    use dialoguer::Confirm;

    let confirmado = Confirm::new()
        .with_prompt(format!(
            "{} ¿Seguro que quieres desinstalar {}?",
            "😥".yellow().bold(),
            "wn++".cyan().bold(),
        ))
        .default(false)
        .interact()
        .into_diagnostic()?;

    if confirmado {
        self_replace::self_delete().into_diagnostic()?;
        println!("{} wn++ desinstalado. Nos vimoooooooo", "✓".green().bold());
    } else {
        println!(
            "{}",
            "Operación cancelada. Cuidadito nomas compare".dimmed()
        );
    }

    Ok(())
}

fn eprint_error(err: WnDiagnostic) {
    eprintln!("{:?}", miette::Report::new(err));
}
