mod common;

use insta::assert_snapshot;
use wn::error::WnError;

use common::{render_error, run_program, run_program_with_output};

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
