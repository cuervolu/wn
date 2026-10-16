use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

/// Proyecto temporal que se borra al terminar.
struct Proyecto(PathBuf);

impl Proyecto {
    fn nuevo(nombre: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("wn-it-{nombre}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir.canonicalize().unwrap())
    }

    fn archivo(&self, ruta: &str, contenido: &str) -> &Self {
        let destino = self.0.join(ruta);
        fs::create_dir_all(destino.parent().unwrap()).unwrap();
        fs::write(destino, contenido).unwrap();
        self
    }

    fn ruta(&self, ruta: &str) -> PathBuf {
        self.0.join(ruta)
    }

    /// Ejecuta `archivo` desde el directorio `cwd`.
    fn correr_desde(&self, archivo: &str, cwd: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_wn"))
            .arg(self.ruta(archivo))
            .current_dir(cwd)
            .env("NO_COLOR", "1")
            .output()
            .unwrap()
    }

    fn correr(&self, archivo: &str) -> Output {
        self.correr_desde(archivo, &self.0)
    }

    /// Lanza el REPL con `lineas` por stdin desde el directorio del proyecto.
    fn repl(&self, lineas: &str) -> Output {
        let mut hijo = Command::new(env!("CARGO_BIN_EXE_wn"))
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        hijo.stdin
            .take()
            .unwrap()
            .write_all(lineas.as_bytes())
            .unwrap();
        hijo.wait_with_output().unwrap()
    }
}

impl Drop for Proyecto {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// Stderr sin los bordes de miette ni saltos de línea, para buscar frases.
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

const MAT: &str =
    "convida pega sumar(a, b) { devolver a + b }\nconvida duro PI = 3\nwea secreto = 1\n";

#[test]
fn importa_modulo_local_subdirectorio_y_alias() {
    let p = Proyecto::nuevo("basico");
    p.archivo("matematicas.cl", MAT)
        .archivo("utilidades/texto.cl", "convida pega saludar(n) { devolver \"hola \" + n }")
        .archivo(
            "main.cl",
            "queri matematicas\nqueri utilidades::texto como t\nlorea(matematicas::sumar(10, 20))\nlorea(t::saludar(\"wn\"))\nlorea(matematicas::PI)",
        );
    let o = p.correr("main.cl");
    assert!(o.status.success(), "{}", stderr(&o));
    assert_eq!(stdout(&o), "30\nhola wn\n3\n");
}

#[test]
fn la_resolucion_no_depende_del_directorio_de_trabajo() {
    let p = Proyecto::nuevo("cwd");
    p.archivo("matematicas.cl", MAT).archivo(
        "main.cl",
        "queri matematicas\nlorea(matematicas::sumar(1, 2))",
    );
    let otro = Proyecto::nuevo("cwd-otro");
    for cwd in [&p.0, &otro.0, &PathBuf::from("/")] {
        let o = p.correr_desde("main.cl", cwd);
        assert!(o.status.success(), "cwd={cwd:?}: {}", stderr(&o));
        assert_eq!(stdout(&o), "3\n");
    }
}

#[test]
fn los_imports_se_resuelven_desde_la_raiz_y_no_desde_el_importador() {
    let p = Proyecto::nuevo("raiz");
    p.archivo(
        "utilidades/a.cl",
        "queri utilidades::b\nconvida wea v = b::v + 1",
    )
    .archivo("utilidades/b.cl", "convida wea v = 1")
    .archivo("main.cl", "queri utilidades::a\nlorea(a::v)");
    let o = p.correr("main.cl");
    assert!(o.status.success(), "{}", stderr(&o));
    assert_eq!(stdout(&o), "2\n");
}

#[test]
fn modulo_inexistente_sugiere_la_ruta_buscada() {
    let p = Proyecto::nuevo("inexistente");
    p.archivo("main.cl", "queri utilidades::fantasma");
    let o = p.correr("main.cl");
    assert!(!o.status.success());
    let err = stderr(&o);
    assert!(err.contains("utilidades::fantasma"), "{err}");
    assert!(err.contains("utilidades/fantasma.cl"), "{err}");
}

#[test]
fn acceso_a_declaracion_privada_sugiere_convida() {
    let p = Proyecto::nuevo("privada");
    p.archivo("matematicas.cl", MAT)
        .archivo("main.cl", "queri matematicas\nlorea(matematicas::secreto)");
    let o = p.correr("main.cl");
    assert!(!o.status.success());
    let err = stderr(&o);
    assert!(err.contains("no es público"), "{err}");
    assert!(err.contains("convida"), "{err}");
}

#[test]
fn ciclo_indirecto_muestra_la_cadena_de_dependencias() {
    let p = Proyecto::nuevo("ciclo");
    p.archivo("a.cl", "queri b")
        .archivo("b.cl", "queri c")
        .archivo("c.cl", "queri a")
        .archivo("main.cl", "queri a");
    let o = p.correr("main.cl");
    assert!(!o.status.success());
    assert!(
        stderr(&o).contains("main.cl → a.cl → b.cl → c.cl → a.cl"),
        "{}",
        stderr(&o)
    );
}

#[test]
fn error_en_modulo_importado_apunta_al_archivo_del_modulo() {
    let p = Proyecto::nuevo("sintaxis");
    p.archivo("roto.cl", "wea = 1")
        .archivo("main.cl", "queri roto");
    let o = p.correr("main.cl");
    assert!(!o.status.success());
    let err = stderr(&o);
    assert!(err.contains("roto.cl"), "{err}");
    assert!(err.contains("main.cl"), "{err}");
}

#[test]
fn conflicto_entre_modulo_local_y_stdlib() {
    let p = Proyecto::nuevo("conflicto");
    p.archivo("cadena.cl", "convida wea x = 1")
        .archivo("main.cl", "queri cadena");
    let o = p.correr("main.cl");
    assert!(!o.status.success());
    let err = stderr(&o);
    assert!(err.contains("ambiguo"), "{err}");
    assert!(err.contains("std::cadena"), "{err}");

    // Con el prefijo reservado, la stdlib siempre es accesible.
    p.archivo("main2.cl", "queri std::cadena\nlorea(cadena::largo)");
    let o = p.correr("main2.cl");
    assert!(!stderr(&o).contains("ambiguo"), "{}", stderr(&o));
}

#[test]
fn std_esta_reservado_para_la_stdlib() {
    let p = Proyecto::nuevo("std");
    p.archivo("std/cadena.cl", "convida wea x = 1")
        .archivo("main.cl", "queri std::inventado");
    let o = p.correr("main.cl");
    assert!(!o.status.success());
    assert!(stderr(&o).contains("reservado"), "{}", stderr(&o));
}

#[cfg(unix)]
#[test]
fn un_symlink_no_puede_sacar_la_resolucion_de_la_raiz() {
    let p = Proyecto::nuevo("symlink");
    let fuera = Proyecto::nuevo("symlink-fuera");
    fuera.archivo("secreto.cl", "convida wea x = 1");
    std::os::unix::fs::symlink(fuera.ruta("secreto.cl"), p.ruta("fuga.cl")).unwrap();
    p.archivo("main.cl", "queri fuga");
    let o = p.correr("main.cl");
    assert!(!o.status.success());
    assert!(stderr(&o).contains("fuera de la raíz"), "{}", stderr(&o));
}

#[test]
fn modulo_importado_por_varios_se_inicializa_una_sola_vez() {
    let p = Proyecto::nuevo("unavez");
    p.archivo("base.cl", "lorea(\"init base\")\nconvida wea v = 1")
        .archivo("a.cl", "queri base")
        .archivo("b.cl", "queri base")
        .archivo("main.cl", "queri base\nqueri a\nqueri b");
    let o = p.correr("main.cl");
    assert!(o.status.success(), "{}", stderr(&o));
    assert_eq!(stdout(&o), "init base\n");
}

// ---------------------------------------------------------------------------
// REPL
// ---------------------------------------------------------------------------

#[test]
fn repl_importa_modulos_locales_y_conserva_el_estado() {
    let p = Proyecto::nuevo("repl");
    p.archivo(
        "c.cl",
        "lorea(\"init\")\nconvida wea n = 0\nconvida pega subir() { n = n + 1 }",
    );
    let o = p.repl("queri c\nc::subir()\nqueri c como otro\notro::subir()\nlorea(c::n)\nchao\n");
    let salida = stdout(&o);
    assert_eq!(salida.matches("init").count(), 1, "{salida}");
    assert!(
        salida.contains("\n2\n") || salida.ends_with("2\n"),
        "{salida}"
    );
}

#[test]
fn repl_permite_reintentar_un_import_fallido_tras_corregir_el_archivo() {
    let p = Proyecto::nuevo("repl-reintento");
    p.archivo("m.cl", "convida wea x = 1 / 0");
    let mut hijo = Command::new(env!("CARGO_BIN_EXE_wn"))
        .current_dir(&p.0)
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Un hilo avisa en cuanto el REPL imprime el primer error por stderr.
    let mut err = hijo.stderr.take().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut acumulado = Vec::new();
        let mut buf = [0u8; 4096];
        while let Ok(n) = err.read(&mut buf) {
            if n == 0 {
                break;
            }
            acumulado.extend_from_slice(&buf[..n]);
            let _ = tx.send(String::from_utf8_lossy(&acumulado).into_owned());
        }
    });

    let mut stdin = hijo.stdin.take().unwrap();
    stdin.write_all(b"queri m\n").unwrap();
    stdin.flush().unwrap();
    let limite = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let mut visto = String::new();
    while !visto.contains("dividir por cero") {
        let resto = limite.saturating_duration_since(std::time::Instant::now());
        visto = rx
            .recv_timeout(resto)
            .unwrap_or_else(|_| panic!("el REPL no reportó el error esperado: {visto}"));
    }

    // El módulo fallido no quedó cacheado: corregido el archivo, el mismo REPL lo importa.
    fs::write(p.ruta("m.cl"), "convida wea x = 8").unwrap();
    stdin.write_all(b"queri m\nlorea(m::x)\nchao\n").unwrap();
    drop(stdin);
    let o = hijo.wait_with_output().unwrap();
    assert!(stdout(&o).contains('8'), "{}", stdout(&o));
}

#[test]
fn el_proyecto_de_ejemplo_documentado_funciona() {
    let ejemplo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/modulos/main.cl");
    let o = Command::new(env!("CARGO_BIN_EXE_wn"))
        .arg(&ejemplo)
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    assert_eq!(stdout(&o), "30\n3.14159\nHola, WN++!\n2\n");
}
