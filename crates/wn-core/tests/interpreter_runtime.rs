mod common;

use insta::assert_snapshot;
use wn::error::WnError;

use common::{render_error, run_program, run_program_with_io, run_program_with_output};

#[test]
fn duro_redeclarado_lanza_error() {
    let resultado = run_program("duro PI = 3.14\nduro PI = 99");

    assert!(resultado.is_err());
    let err = resultado.unwrap_err();
    assert!(matches!(err, WnError::ConstanteInmutable { .. }));
    assert_snapshot!("duro_redeclarado_lanza_error", render_error(&err));
}

#[test]
fn duro_reasignacion_directa_lanza_error() {
    let resultado = run_program("duro PI = 3.14\nPI = 99");

    assert!(resultado.is_err());
    let err = resultado.unwrap_err();
    assert!(matches!(err, WnError::ConstanteInmutable { .. })); // ← antes: Runtime
    assert_snapshot!("duro_reasignacion_directa_lanza_error", render_error(&err));
}

#[test]
fn duro_valor_no_cambia_tras_intento_fallido() {
    let src = "duro PI = 3.14\nojo { PI = 99 } cago(err) { lorea(PI) }";
    let (resultado, stdout) = run_program_with_output(src);

    assert!(resultado.is_ok());
    assert_snapshot!("duro_valor_no_cambia_tras_intento_fallido", stdout);
}

#[test]
fn wea_puede_cambiar_de_tipo() {
    let (resultado, stdout) = run_program_with_output("wea x = 10\nx = \"hola\"\nlorea(x)");

    assert!(resultado.is_ok());
    assert_snapshot!("wea_puede_cambiar_de_tipo", stdout);
}

#[test]
fn variable_no_definida_da_error_correcto() {
    let resultado = run_program("lorea(x_que_no_existe)");

    assert!(resultado.is_err());
    let err = resultado.unwrap_err();
    assert!(matches!(err, WnError::VarNoDefinida { .. }));
    assert_snapshot!("variable_no_definida_da_error_correcto", render_error(&err));
}

#[test]
fn indices_numericos_aceptan_enteros_positivos_y_negativos() {
    let (resultado, stdout) = run_program_with_output(
        r#"
        lorea([10, 20, 30][1])
        lorea("hola"[1])
        lorea([10, 20, 30][-1])
        lorea("hola"[-1])
    "#,
    );

    assert!(resultado.is_ok());
    assert_eq!(stdout, "20\no\n30\na\n");
}

#[test]
fn indices_numericos_rechazan_fraccionarios_en_lista_y_texto() {
    let lista = run_program("[10, 20, 30][1.9]");
    let texto = run_program(r#""hola"[1.1]"#);

    assert!(matches!(lista, Err(WnError::TipoInvalido { .. })));
    assert!(matches!(texto, Err(WnError::TipoInvalido { .. })));
}

#[test]
fn indices_numericos_rechazan_no_finitos() {
    let gigante = "9".repeat(400);
    let infinito = run_program(&format!("[10, 20, 30][{gigante}]"));
    let nan = run_program(&format!(
        "wea gigante = {gigante}\nlorea([10, 20, 30][gigante - gigante])"
    ));

    assert!(matches!(infinito, Err(WnError::TipoInvalido { .. })));
    assert!(matches!(nan, Err(WnError::TipoInvalido { .. })));
}

#[test]
fn indices_numericos_rechazan_enteros_fuera_de_rango_i64() {
    let positivo = run_program("[10, 20, 30][9223372036854775808]");
    let negativo = run_program("[10, 20, 30][-9223372036854775809]");

    assert!(matches!(positivo, Err(WnError::TipoInvalido { .. })));
    assert!(matches!(negativo, Err(WnError::TipoInvalido { .. })));
}

#[test]
fn numero_convierte_textos_y_numeros_validos() {
    let src = r#"
    lorea(numero("42"))
    lorea(numero("3.14"))
    lorea(numero("-7"))
    lorea(numero(" 19 "))
    lorea(numero(8))
"#;

    let (resultado, stdout) = run_program_with_output(src);

    assert!(resultado.is_ok());
    assert_snapshot!(stdout);
}

#[test]
fn numero_rechaza_texto_no_convertible() {
    let resultado = run_program(r#"numero("hola")"#);

    assert!(resultado.is_err());
    let err = resultado.unwrap_err();
    assert!(matches!(err, WnError::TextoNoConvertibleANumero { .. }));
    assert_snapshot!(render_error(&err));
}

#[test]
fn numero_rechaza_texto_vacio_nan_e_infinity() {
    let src = r#"
    ojo { numero("") } cago(e) { lorea(e) }
    ojo { numero("NaN") } cago(e) { lorea(e) }
    ojo { numero("Infinity") } cago(e) { lorea(e) }
"#;

    let (resultado, stdout) = run_program_with_output(src);

    assert!(resultado.is_ok());
    assert_snapshot!(stdout);
}

#[test]
fn numero_rechaza_tipos_no_convertibles() {
    let resultado = run_program("numero(verdad)");

    assert!(resultado.is_err());
    let err = resultado.unwrap_err();
    assert!(matches!(err, WnError::TipoInvalido { .. }));
    assert_snapshot!(render_error(&err));
}

#[test]
fn numero_rechaza_aridad_invalida() {
    let sin_args = run_program("numero()");
    let muchos_args = run_program(r#"numero("1", "2")"#);

    assert!(sin_args.is_err());
    let err = sin_args.unwrap_err();
    assert!(matches!(err, WnError::NumArgInvalido { .. }));
    assert_snapshot!(
        "numero_rechaza_aridad_invalida_sin_args",
        render_error(&err)
    );

    assert!(muchos_args.is_err());
    let err = muchos_args.unwrap_err();
    assert!(matches!(err, WnError::NumArgInvalido { .. }));
    assert_snapshot!(
        "numero_rechaza_aridad_invalida_muchos_args",
        render_error(&err)
    );
}

#[test]
fn numero_rechaza_formatos_no_soportados() {
    let src = r#"
ojo { numero("1e3") } cago(e) { lorea(e) }
ojo { numero("3,14") } cago(e) { lorea(e) }
ojo { numero("+7") } cago(e) { lorea(e) }
ojo { numero("1.2.3") } cago(e) { lorea(e) }
ojo { numero(".5") } cago(e) { lorea(e) }
ojo { numero("5.") } cago(e) { lorea(e) }
"#;

    let (resultado, stdout) = run_program_with_output(src);

    assert!(resultado.is_ok());
    assert_snapshot!(stdout);
}

#[test]
fn texto_convierte_cualquier_valor() {
    let src = r#"
lorea(texto("hola"))
    lorea(texto(42))
    lorea(texto(3.5))
    lorea(texto(verdad))
    lorea(texto(falso))
    lorea(texto(nada))
    lorea(texto([1, "hola"]))
"#;

    let (resultado, stdout) = run_program_with_output(src);

    assert!(resultado.is_ok());
    assert_snapshot!(stdout);
}

#[test]
fn texto_rechaza_aridad_invalida() {
    let sin_args = run_program("texto()");
    let muchos_args = run_program("texto(1, 2)");

    assert!(sin_args.is_err());
    let err = sin_args.unwrap_err();
    assert!(matches!(err, WnError::NumArgInvalido { .. }));
    assert_snapshot!("texto_rechaza_aridad_invalida_sin_args", render_error(&err));

    assert!(muchos_args.is_err());
    let err = muchos_args.unwrap_err();
    assert!(matches!(err, WnError::NumArgInvalido { .. }));
    assert_snapshot!(
        "texto_rechaza_aridad_invalida_muchos_args",
        render_error(&err)
    );
}

#[test]
fn numero_error_es_capturable_con_ojo_y_cago() {
    let src = r#"
ojo {
      numero("papas")
    } cago(e) {
      lorea("error: " + e)
    }
"#;

    let (resultado, stdout) = run_program_with_output(src);

    assert!(resultado.is_ok());
    assert_snapshot!(stdout);
}

#[test]
fn numero_funciona_con_pregunta() {
    let src = r#"
    wea edad = numero(pregunta("Edad: "))
    lorea(edad + 1)
"#;

    let (resultado, stdout) = run_program_with_io(src, "41\n");

    assert!(resultado.is_ok());
    assert_snapshot!(stdout);
}
