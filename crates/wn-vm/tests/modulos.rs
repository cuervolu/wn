use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Cursor, Write},
    rc::Rc,
    sync::Arc,
};

use wn::{lexer::tokenizar, parser::parsear};
use wn_diagnostics::{SourceFile, WnDiagnostic};
use wn_vm::{
    compiler::{compilar, compilar_repl},
    resolver::{CompositeResolver, ErrorResolucion, ModuleResolver, ModuloResuelto},
    value::Value,
    vm::VM,
};

type Fuentes = Rc<RefCell<HashMap<String, String>>>;

/// Sirve módulos desde memoria. `alias` permite que dos paths distintos apunten
/// al mismo módulo (misma identidad), como dos rutas al mismo archivo.
struct MemResolver {
    fuentes: Fuentes,
    alias: HashMap<String, String>,
}

impl ModuleResolver for MemResolver {
    fn resolver(&self, path: &[&str]) -> Result<Option<ModuloResuelto>, ErrorResolucion> {
        let clave = path.join("::");
        let id = self.alias.get(&clave).cloned().unwrap_or(clave);
        Ok(self
            .fuentes
            .borrow()
            .get(&id)
            .map(|src| ModuloResuelto::Fuente {
                id: id.clone(),
                source: Arc::new(SourceFile::new(
                    format!("{}.cl", id.replace("::", "/")),
                    src.clone(),
                )),
            }))
    }
}

/// Resolver que siempre responde con un módulo nativo (para probar conflictos).
struct Nativo(&'static str);

impl ModuleResolver for Nativo {
    fn resolver(&self, path: &[&str]) -> Result<Option<ModuloResuelto>, ErrorResolucion> {
        Ok((path == [self.0])
            .then(|| ModuloResuelto::Nativo(Value::Modulo(Rc::new(HashMap::new())))))
    }
}

#[derive(Clone, Default)]
struct Salida(Rc<RefCell<Vec<u8>>>);

impl Write for Salida {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct Entorno {
    vm: VM,
    fuentes: Fuentes,
    salida: Salida,
}

fn entorno(modulos: &[(&str, &str)]) -> Entorno {
    entorno_con(modulos, &[])
}

fn entorno_con(modulos: &[(&str, &str)], alias: &[(&str, &str)]) -> Entorno {
    let fuentes: Fuentes = Rc::new(RefCell::new(
        modulos
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    ));
    let resolver = MemResolver {
        fuentes: fuentes.clone(),
        alias: alias
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    };
    let salida = Salida::default();
    let vm = VM::con_resolver(
        salida.clone(),
        Cursor::new(Vec::<u8>::new()),
        Box::new(resolver),
    );
    Entorno {
        vm,
        fuentes,
        salida,
    }
}

impl Entorno {
    /// Ejecuta como script de entrada (nombre `main.cl`).
    fn main(&mut self, src: &str) -> Result<Value, WnDiagnostic> {
        let stmts = parsear(tokenizar(src).unwrap(), src, "main.cl").unwrap();
        let chunk = compilar(&stmts, Arc::new(SourceFile::new("main.cl", src))).unwrap();
        self.vm.run(&chunk)
    }

    /// Ejecuta una línea como el REPL (misma VM, estado persistente).
    fn repl(&mut self, src: &str) -> Result<Value, WnDiagnostic> {
        let stmts = parsear(tokenizar(src).unwrap(), src, "<repl>").unwrap();
        let chunk = compilar_repl(&stmts, Arc::new(SourceFile::new("<repl>", src))).unwrap();
        self.vm.run(&chunk)
    }

    fn salida(&self) -> String {
        String::from_utf8(self.salida.0.borrow().clone()).unwrap()
    }

    fn lineas(&self) -> Vec<String> {
        self.salida().lines().map(str::to_string).collect()
    }
}

fn causa(d: &WnDiagnostic) -> &WnDiagnostic {
    match d {
        WnDiagnostic::ErrorEnModulo { causa, .. } => &causa.0,
        otro => otro,
    }
}

// ---------------------------------------------------------------------------
// Importación, alias, nativos
// ---------------------------------------------------------------------------

#[test]
fn importa_modulo_local() {
    let mut e = entorno(&[("mat", "convida pega sumar(a, b) { devolver a + b }")]);
    e.main("queri mat\nlorea(mat::sumar(10, 20))").unwrap();
    assert_eq!(e.lineas(), ["30"]);
}

#[test]
fn importa_desde_subdirectorio_con_el_ultimo_segmento_como_nombre() {
    let mut e = entorno(&[("utilidades::texto", "convida wea x = 5")]);
    e.main("queri utilidades::texto\nlorea(texto::x)").unwrap();
    assert_eq!(e.lineas(), ["5"]);
}

#[test]
fn importa_con_alias() {
    let mut e = entorno(&[("utilidades::texto", "convida wea x = 5")]);
    e.main("queri utilidades::texto como t\nlorea(t::x)")
        .unwrap();
    assert_eq!(e.lineas(), ["5"]);
}

#[test]
fn modulos_nativos_siguen_funcionando_junto_a_los_locales() {
    let fuentes: Fuentes = Rc::new(RefCell::new(HashMap::from([(
        "mat".to_string(),
        "convida wea x = 1".to_string(),
    )])));
    let resolver = CompositeResolver::new(vec![
        Box::new(Nativo("nat")),
        Box::new(MemResolver {
            fuentes,
            alias: HashMap::new(),
        }),
    ]);
    let mut vm = VM::con_resolver(
        Salida::default(),
        Cursor::new(Vec::<u8>::new()),
        Box::new(resolver),
    );
    let src = "queri nat\nqueri mat";
    let stmts = parsear(tokenizar(src).unwrap(), src, "main.cl").unwrap();
    let chunk = compilar(&stmts, Arc::new(SourceFile::new("main.cl", src))).unwrap();
    vm.run(&chunk).unwrap();
    assert!(matches!(vm.globals["nat"], Value::Modulo(_)));
    assert!(matches!(vm.globals["mat"], Value::ModuloUsuario(_)));
}

#[test]
fn nativo_y_local_con_el_mismo_nombre_es_conflicto() {
    let fuentes: Fuentes = Rc::new(RefCell::new(HashMap::from([(
        "cadena".to_string(),
        "convida wea x = 1".to_string(),
    )])));
    let resolver = CompositeResolver::new(vec![
        Box::new(Nativo("cadena")),
        Box::new(MemResolver {
            fuentes,
            alias: HashMap::new(),
        }),
    ]);
    let mut vm = VM::con_resolver(
        Salida::default(),
        Cursor::new(Vec::<u8>::new()),
        Box::new(resolver),
    );
    let src = "queri cadena";
    let stmts = parsear(tokenizar(src).unwrap(), src, "main.cl").unwrap();
    let chunk = compilar(&stmts, Arc::new(SourceFile::new("main.cl", src))).unwrap();
    let err = vm.run(&chunk).unwrap_err();
    assert!(
        matches!(err, WnDiagnostic::ModuloConflicto { .. }),
        "{err:?}"
    );
}

#[test]
fn un_modulo_puede_tapar_una_builtin_dentro_del_archivo_que_lo_importa() {
    let mut e = entorno(&[("utilidades::texto", "convida wea x = 5")]);
    e.main("queri utilidades::texto\nlorea(texto::x)").unwrap();
    assert_eq!(e.lineas(), ["5"]);
}

#[test]
fn importacion_selectiva_sigue_importando_submodulos() {
    let mut e = entorno(&[
        ("util::a", "convida wea v = 1"),
        ("util::b", "convida wea v = 2"),
    ]);
    e.main("queri util::{a, b}\nlorea(a::v)\nlorea(b::v)")
        .unwrap();
    assert_eq!(e.lineas(), ["1", "2"]);
}

// ---------------------------------------------------------------------------
// Exportaciones y visibilidad
// ---------------------------------------------------------------------------

const MAT: &str = "
convida duro PI = 3
convida wea contador = 0
wea privada = 1
wea secreto = 42
pega interna() { devolver secreto }
convida pega sumar(a, b) { devolver a + b }
convida pega leer_secreto() { devolver interna() }
";

#[test]
fn exporta_funciones_variables_y_constantes() {
    let mut e = entorno(&[("mat", MAT)]);
    e.main("queri mat\nlorea(mat::sumar(1, 2))\nlorea(mat::PI)\nlorea(mat::contador)\nlorea(mat::leer_secreto())")
        .unwrap();
    assert_eq!(e.lineas(), ["3", "3", "0", "42"]);
}

#[test]
fn declaraciones_privadas_no_son_accesibles() {
    let mut e = entorno(&[("mat", MAT)]);
    for privado in ["secreto", "interna"] {
        let err = e.main(&format!("queri mat\nmat::{privado}")).unwrap_err();
        let WnDiagnostic::NoExportado { nombre, .. } = causa(&err) else {
            panic!("se esperaba NoExportado, llegó {err:?}");
        };
        assert_eq!(&**nombre, privado);
        // Cada intento usa una VM limpia para no chocar con el binding `mat`.
        e = entorno(&[("mat", MAT)]);
    }
}

#[test]
fn exportacion_inexistente_se_distingue_de_la_privada() {
    let mut e = entorno(&[("mat", MAT)]);
    let err = e.main("queri mat\nmat::nada_de_nada").unwrap_err();
    assert!(
        matches!(err, WnDiagnostic::ExportInexistente { .. }),
        "{err:?}"
    );
}

#[test]
fn convida_dentro_de_un_bloque_es_error_de_sintaxis() {
    let src = "pega f() { convida wea x = 1 }";
    assert!(parsear(tokenizar(src).unwrap(), src, "main.cl").is_err());
}

// ---------------------------------------------------------------------------
// Aislamiento y closures
// ---------------------------------------------------------------------------

#[test]
fn los_globales_de_un_modulo_no_contaminan_al_importador() {
    let mut e = entorno(&[("mat", "wea oculto = 1\nconvida wea publico = 2")]);
    e.main("queri mat").unwrap();
    assert!(!e.vm.globals.contains_key("oculto"));
    assert!(!e.vm.globals.contains_key("publico"));
}

#[test]
fn un_modulo_no_ve_los_globales_del_script_de_entrada() {
    let mut e = entorno(&[("mat", "convida pega ver() { devolver del_main }")]);
    let err = e
        .main("wea del_main = 1\nqueri mat\nmat::ver()")
        .unwrap_err();
    assert!(matches!(err, WnDiagnostic::VarNoDefinida { .. }), "{err:?}");
}

#[test]
fn dos_modulos_pueden_usar_el_mismo_nombre_global_sin_pisarse() {
    let mut e = entorno(&[
        ("a", "wea x = 1\nconvida pega ver() { devolver x }"),
        ("b", "wea x = 2\nconvida pega ver() { devolver x }"),
    ]);
    e.main("wea x = 3\nqueri a\nqueri b\nlorea(a::ver())\nlorea(b::ver())\nlorea(x)")
        .unwrap();
    assert_eq!(e.lineas(), ["1", "2", "3"]);
}

#[test]
fn una_closure_conserva_el_modulo_donde_fue_definida() {
    // `b` llama una función de `a`; esa función debe ver el `x` de `a`, no el de `b`.
    let mut e = entorno(&[
        (
            "a",
            "wea x = 10\nconvida pega ver() { devolver x }\nconvida pega fabrica() { devolver ver }",
        ),
        (
            "b",
            "queri a\nwea x = 20\nconvida pega probar() { wea f = a::fabrica()\n devolver f() }",
        ),
    ]);
    e.main("queri b\nlorea(b::probar())").unwrap();
    assert_eq!(e.lineas(), ["10"]);
}

#[test]
fn una_closure_pasada_como_valor_sigue_viendo_su_modulo() {
    let mut e = entorno(&[("a", "wea x = 7\nconvida pega ver() { devolver x }")]);
    // Desde el script de entrada existe otro `x`; no debe filtrarse a la función de `a`.
    e.main("wea x = 99\nqueri a\nwea f = a::ver\nlorea(f())")
        .unwrap();
    assert_eq!(e.lineas(), ["7"]);
}

// ---------------------------------------------------------------------------
// Variables mutables exportadas
// ---------------------------------------------------------------------------

const CONTADOR: &str = "
convida wea cuenta = 0
convida wea lista = [1, 2, 3]
convida pega subir() { cuenta = cuenta + 1 }
convida pega reasignar() { lista = [9] }
convida pega primero() { devolver lista[0] }
";

#[test]
fn reasignaciones_internas_son_visibles_desde_afuera() {
    let mut e = entorno(&[("c", CONTADOR)]);
    e.main("queri c\nc::subir()\nc::subir()\nlorea(c::cuenta)\nc::reasignar()\nlorea(c::lista[0])")
        .unwrap();
    assert_eq!(e.lineas(), ["2", "9"]);
}

#[test]
fn el_importador_puede_mutar_el_contenido_de_una_lista_exportada() {
    let mut e = entorno(&[("c", CONTADOR)]);
    e.main("queri c\nc::lista[0] = 100\nlorea(c::primero())")
        .unwrap();
    assert_eq!(e.lineas(), ["100"]);
}

#[test]
fn el_importador_no_puede_reasignar_el_binding_exportado() {
    let mut e = entorno(&[("c", CONTADOR)]);
    let src = "queri c\nc::cuenta = 5";
    let parsed = parsear(tokenizar(src).unwrap(), src, "main.cl");
    let rechazado = match parsed {
        Err(_) => true,
        Ok(stmts) => compilar(&stmts, Arc::new(SourceFile::new("main.cl", src))).is_err(),
    };
    assert!(
        rechazado,
        "asignar a un binding de otro módulo debe rechazarse"
    );
    e.main("queri c\nlorea(c::cuenta)").unwrap();
    assert_eq!(e.lineas(), ["0"]);
}

// ---------------------------------------------------------------------------
// Caché e inicialización única
// ---------------------------------------------------------------------------

#[test]
fn un_modulo_se_inicializa_una_sola_vez_aunque_varios_lo_importen() {
    let mut e = entorno(&[
        ("base", "lorea(\"init base\")\nconvida wea valor = 1"),
        ("a", "queri base\nconvida wea de_a = base::valor"),
        ("b", "queri base\nconvida wea de_b = base::valor"),
    ]);
    e.main("queri base\nqueri a\nqueri b\nqueri base").unwrap();
    assert_eq!(e.lineas(), ["init base"]);
}

#[test]
fn importadores_distintos_comparten_estado() {
    let mut e = entorno(&[
        ("c", CONTADOR),
        ("a", "queri c\nconvida pega tocar() { c::subir() }"),
        ("b", "queri c\nconvida pega leer() { devolver c::cuenta }"),
    ]);
    e.main("queri a\nqueri b\na::tocar()\na::tocar()\nlorea(b::leer())")
        .unwrap();
    assert_eq!(e.lineas(), ["2"]);
}

#[test]
fn rutas_distintas_a_un_mismo_modulo_comparten_instancia() {
    let mut e = entorno_con(
        &[("real", "lorea(\"init\")\nconvida wea n = 1")],
        &[("atajo", "real"), ("otro::camino", "real")],
    );
    e.main("queri real\nqueri atajo\nqueri otro::camino como camino\nlorea(real::n + atajo::n + camino::n)")
        .unwrap();
    assert_eq!(e.lineas(), ["init", "3"]);
}

#[test]
fn reimportar_el_mismo_modulo_con_el_mismo_nombre_es_inofensivo() {
    let mut e = entorno(&[("mat", "lorea(\"init\")\nconvida wea x = 1")]);
    e.repl("queri mat").unwrap();
    e.repl("queri mat").unwrap();
    assert_eq!(e.lineas(), ["init"]);
}

#[test]
fn importar_con_un_nombre_ya_usado_falla_sin_ejecutar_el_modulo() {
    let mut e = entorno(&[("mat", "lorea(\"init\")")]);
    let err = e.main("wea mat = 1\nqueri mat").unwrap_err();
    assert!(matches!(err, WnDiagnostic::NombreEnUso { .. }), "{err:?}");
    assert!(e.lineas().is_empty(), "el módulo no debió ejecutarse");
}

// ---------------------------------------------------------------------------
// Ciclos
// ---------------------------------------------------------------------------

#[test]
fn ciclo_directo_muestra_la_cadena() {
    let mut e = entorno(&[("a", "queri b"), ("b", "queri a")]);
    let err = e.main("queri a").unwrap_err();
    let WnDiagnostic::ErrorEnModulo { causa: c, .. } = &err else {
        panic!("{err:?}");
    };
    // La cadena completa vive en el diagnóstico de ciclo.
    let WnDiagnostic::ImportCircular { cadena, .. } = &*c.0 else {
        panic!("{c:?}");
    };
    assert_eq!(&**cadena, "main.cl → a.cl → b.cl → a.cl");
}

#[test]
fn ciclo_indirecto_muestra_la_cadena() {
    let mut e = entorno(&[("a", "queri b"), ("b", "queri c"), ("c", "queri a")]);
    let err = e.main("queri a").unwrap_err();
    let WnDiagnostic::ImportCircular { cadena, .. } = causa(&err) else {
        panic!("{err:?}");
    };
    assert_eq!(&**cadena, "main.cl → a.cl → b.cl → c.cl → a.cl");
}

#[test]
fn importarse_a_si_mismo_es_ciclo() {
    let mut e = entorno(&[("a", "queri a")]);
    let err = e.main("queri a").unwrap_err();
    assert!(
        matches!(causa(&err), WnDiagnostic::ImportCircular { .. }),
        "{err:?}"
    );
}

#[test]
fn un_modulo_que_importa_al_script_de_entrada_es_ciclo() {
    let mut e = entorno(&[("a", "queri main")]);
    e.fuentes
        .borrow_mut()
        .insert("main".into(), "queri a".into());
    e.vm.marcar_entrada("main");
    let err = e.main("queri a").unwrap_err();
    let WnDiagnostic::ImportCircular { cadena, .. } = causa(&err) else {
        panic!("{err:?}");
    };
    assert_eq!(&**cadena, "main.cl → a.cl → main.cl");
}

// ---------------------------------------------------------------------------
// Errores y diagnósticos
// ---------------------------------------------------------------------------

#[test]
fn modulo_inexistente() {
    let mut e = entorno(&[]);
    let err = e.main("queri fantasma").unwrap_err();
    assert!(
        matches!(err, WnDiagnostic::ModuloNoEncontrado { .. }),
        "{err:?}"
    );
}

#[test]
fn error_de_sintaxis_en_un_modulo_apunta_al_archivo_del_modulo() {
    let mut e = entorno(&[("roto", "wea = 1")]);
    let err = e.main("queri roto").unwrap_err();
    let WnDiagnostic::ErrorEnModulo { cadena, .. } = &err else {
        panic!("{err:?}");
    };
    assert_eq!(&**cadena, "main.cl → roto.cl");
    assert!(matches!(causa(&err), WnDiagnostic::Sintaxis { .. }));
}

#[test]
fn error_de_compilacion_en_un_modulo() {
    let mut e = entorno(&[("roto", "devolver 1")]);
    let err = e.main("queri roto").unwrap_err();
    assert!(
        matches!(causa(&err), WnDiagnostic::Compilacion { .. }),
        "{err:?}"
    );
}

#[test]
fn error_de_ejecucion_en_un_modulo_incluye_la_cadena() {
    let mut e = entorno(&[("a", "queri b"), ("b", "wea x = 1 / 0")]);
    let err = e.main("queri a").unwrap_err();
    let WnDiagnostic::ErrorEnModulo { cadena, .. } = &err else {
        panic!("{err:?}");
    };
    assert_eq!(&**cadena, "main.cl → a.cl → b.cl");
    assert!(matches!(causa(&err), WnDiagnostic::DivisionPorCero { .. }));
}

// ---------------------------------------------------------------------------
// Fallos de inicialización y reintentos
// ---------------------------------------------------------------------------

#[test]
fn falla_dependencia_intermedia_y_ningun_modulo_queda_en_cache() {
    let mut e = entorno(&[
        ("a", "lorea(\"init a\")\nqueri b"),
        ("b", "lorea(\"init b\")\nqueri c"),
        ("c", "lorea(\"init c\")\nwea x = 1 / 0"),
    ]);
    assert!(e.main("queri a").is_err());
    assert_eq!(e.lineas(), ["init a", "init b", "init c"]);
    for nombre in ["a", "b", "c"] {
        assert!(!e.vm.globals.contains_key(nombre));
    }

    // Se arregla `c` y se reintenta con la misma VM: todo se inicializa de nuevo.
    e.fuentes
        .borrow_mut()
        .insert("c".into(), "lorea(\"init c ok\")".into());
    e.repl("queri a").unwrap();
    assert_eq!(
        e.lineas(),
        [
            "init a",
            "init b",
            "init c",
            "init a",
            "init b",
            "init c ok"
        ]
    );
    assert!(matches!(e.vm.globals["a"], Value::ModuloUsuario(_)));
}

#[test]
fn falla_la_dependencia_final_y_los_modulos_previos_no_quedan_a_medias() {
    // `b` termina bien dentro de `a` y queda cacheado; `a` falló, así que no.
    // Tras corregir `c`, `a` se inicializa de cero pero `b` no se reinicializa.
    let mut e = entorno(&[
        ("a", "queri b\nqueri c"),
        ("b", "lorea(\"init b\")\nconvida wea v = 1"),
        ("c", "wea x = 1 / 0"),
    ]);
    assert!(e.main("queri a").is_err());
    e.fuentes
        .borrow_mut()
        .insert("c".into(), "convida wea v = 2".into());
    e.repl("queri a").unwrap();
    // `b` ya estaba bien cargado: no se reinicializa.
    assert_eq!(e.lineas(), ["init b"]);
}

#[test]
fn reintento_desde_el_repl_tras_corregir_el_modulo() {
    let mut e = entorno(&[("m", "convida wea x = 1 / 0")]);
    assert!(e.repl("queri m").is_err());
    assert!(!e.vm.globals.contains_key("m"));
    e.fuentes
        .borrow_mut()
        .insert("m".into(), "convida wea x = 8".into());
    e.repl("queri m").unwrap();
    e.repl("lorea(m::x)").unwrap();
    assert_eq!(e.lineas(), ["8"]);
}

#[test]
fn un_import_fallido_se_puede_capturar_con_ojo() {
    let mut e = entorno(&[("m", "wea x = 1 / 0")]);
    e.main("ojo {\n queri m\n} cago (err) {\n lorea(\"capturado\")\n}\nlorea(\"sigue\")")
        .unwrap();
    assert_eq!(e.lineas(), ["capturado", "sigue"]);
    // Y el módulo fallido tampoco quedó cacheado: arreglarlo permite importarlo.
    e.fuentes
        .borrow_mut()
        .insert("m".into(), "convida wea x = 3".into());
    e.repl("queri m").unwrap();
    e.repl("lorea(m::x)").unwrap();
    assert_eq!(e.lineas().last().unwrap(), "3");
}

// ---------------------------------------------------------------------------
// REPL
// ---------------------------------------------------------------------------

#[test]
fn el_repl_conserva_imports_y_cache_entre_lineas() {
    let mut e = entorno(&[("c", &format!("lorea(\"init\")\n{CONTADOR}"))]);
    e.repl("queri c").unwrap();
    e.repl("c::subir()").unwrap();
    e.repl("queri c como otro").unwrap();
    e.repl("otro::subir()").unwrap();
    let v = e.repl("c::cuenta").unwrap();
    assert_eq!(v, Value::Numero(2.0));
    assert_eq!(e.lineas(), ["init"]);
}
