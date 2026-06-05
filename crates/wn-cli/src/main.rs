mod ast_dot;
mod updater;

use clap::Parser;
use miette::IntoDiagnostic;
use owo_colors::OwoColorize;
use rustyline::{DefaultEditor, error::ReadlineError};
use std::io::BufReader;
use std::{collections::HashSet, io, path::Path, path::PathBuf, rc::Rc, sync::Arc};
use wn::{
    ast::Stmt,
    lexer::{Lexer, tokenizar},
    parser::parsear,
};
use wn_diagnostics::{SourceFile, WnDiagnostic};
use wn_stdlib::stdlib_resolver::StdlibResolver;
use wn_vm::resolver::CompositeResolver;
use wn_vm::{
    chunk::Chunk,
    compiler::{compilar, compilar_repl},
    value::{ObjFunction, Value},
    vm::VM,
};

use crate::ast_dot::ast_to_dot;
use crate::updater::run_update;

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
    /// Exporta el AST del programa a Graphviz (.dot)
    Ast {
        /// Archivo fuente a parsear (.cl)
        file: PathBuf,

        /// Escribe la salida DOT a un archivo en vez de stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Desensambla el bytecode compilado del programa
    Chunk {
        /// Archivo fuente a compilar (.cl)
        file: PathBuf,
    },
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
        Some(Command::Ast { file, output }) => run_ast(file, output),
        Some(Command::Chunk { file }) => run_chunk(file),
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
    let loaded = load_source_file(&path)?;
    let stmts = parse_source(&loaded)?;
    let chunk = compile_stmts(&loaded, &stmts)?;
    let mut vm = crear_vm();
    let _ = vm.run(&chunk)?;
    Ok(())
}

fn run_ast(path: PathBuf, output: Option<PathBuf>) -> miette::Result<()> {
    let loaded = load_source_file(&path)?;
    let stmts = parse_source(&loaded)?;
    let dot = ast_to_dot(&stmts);

    if let Some(output) = output {
        std::fs::write(output, dot).into_diagnostic()?;
    } else {
        print!("{dot}");
    }

    Ok(())
}

fn run_chunk(path: PathBuf) -> miette::Result<()> {
    let loaded = load_source_file(&path)?;
    let stmts = parse_source(&loaded)?;
    let chunk = compile_stmts(&loaded, &stmts)?;
    print!("{}", disassemble_chunk_tree(&chunk));
    Ok(())
}

fn crear_vm() -> VM {
    VM::con_resolver(
        io::stdout(),
        BufReader::new(io::stdin()),
        Box::new(CompositeResolver::new(vec![Box::new(StdlibResolver)])),
    )
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

    let mut vm = crear_vm();

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

struct LoadedSource {
    filename: String,
    src: String,
    source: Arc<SourceFile>,
}

fn load_source_file(path: &Path) -> miette::Result<LoadedSource> {
    ensure_cl_file(path)?;

    let src = std::fs::read_to_string(path).into_diagnostic()?;
    let filename = path.to_string_lossy().into_owned();
    let source = Arc::new(SourceFile::new(filename.clone(), src.clone()));

    Ok(LoadedSource {
        filename,
        src,
        source,
    })
}

fn ensure_cl_file(path: &Path) -> miette::Result<()> {
    if path.extension().and_then(|e| e.to_str()) != Some("cl") {
        miette::bail!("se esperaba un archivo .cl, encontré '{}'", path.display());
    }
    Ok(())
}

fn parse_source(loaded: &LoadedSource) -> Result<Vec<Stmt>, WnDiagnostic> {
    let tokens = Lexer::new(&loaded.src)
        .with_filename(&loaded.filename)
        .tokenizar()?;
    parsear(tokens, &loaded.src, &loaded.filename)
}

fn compile_stmts(loaded: &LoadedSource, stmts: &[Stmt]) -> Result<Chunk, WnDiagnostic> {
    compilar(stmts, Arc::clone(&loaded.source))
}

fn disassemble_chunk_tree(root: &Chunk) -> String {
    let mut out = String::new();
    let mut visited = HashSet::new();
    append_chunk_recursive(root, &mut out, &mut visited);
    out
}

fn append_chunk_recursive(chunk: &Chunk, out: &mut String, visited: &mut HashSet<usize>) {
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&chunk.to_string());

    for function in nested_functions(chunk) {
        let key = Rc::as_ptr(&function) as usize;
        if visited.insert(key) {
            append_chunk_recursive(&function.chunk, out, visited);
        }
    }
}

fn nested_functions(chunk: &Chunk) -> Vec<Rc<ObjFunction>> {
    chunk
        .constants
        .iter()
        .filter_map(|value| match value {
            Value::Funcion(function) => Some(Rc::clone(function)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use wn_diagnostics::SourceFile;

    use super::{
        LoadedSource, compile_stmts, disassemble_chunk_tree, nested_functions, parse_source,
    };

    fn loaded(src: &str) -> LoadedSource {
        LoadedSource {
            filename: "<test>".to_string(),
            src: src.to_string(),
            source: Arc::new(SourceFile::new("<test>", src)),
        }
    }

    #[test]
    fn disassemble_chunk_tree_includes_nested_functions() {
        let loaded = loaded("pega afuera(x) { pega adentro() { devolver x }\n devolver adentro }");
        let stmts = parse_source(&loaded).unwrap();
        let chunk = compile_stmts(&loaded, &stmts).unwrap();
        let output = disassemble_chunk_tree(&chunk);

        assert!(output.contains("══ <script> ══"));
        assert!(output.contains("══ afuera ══"));
        assert!(output.contains("══ adentro ══"));
    }

    #[test]
    fn nested_functions_returns_functions_from_constants() {
        let loaded = loaded("pega sumar(a, b) { devolver a + b }");
        let stmts = parse_source(&loaded).unwrap();
        let chunk = compile_stmts(&loaded, &stmts).unwrap();
        let functions = nested_functions(&chunk);

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].nombre, "sumar");
    }
}
