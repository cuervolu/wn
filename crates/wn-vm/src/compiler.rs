//! Compilador de AST a bytecode.
//!
//! Recorre el AST del crate `wn` y emite instrucciones en un [`Chunk`].
//! Un [`Compiler`] nuevo por cada función compilada, el script raíz
//! usa `Compiler::new("<script>")`.

use wn::ast::{Expr, OpBin, OpUn, Stmt};

use crate::{chunk::Chunk, opcode::OpCode, value::Value};

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("No se puede reasignar la constante '{0}', wn.")]
    AsignacionConstante(String),

    #[error("Demasiados argumentos en la llamada (máximo 255).")]
    DemasiadosArgumentos,
}

/// Variable local rastreada en compile-time.
struct Local {
    nombre: String,
    /// Profundidad de scope al momento de definirse.
    depth: u32,
    es_duro: bool,
}

pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: u32,
    duro_globals: std::collections::HashSet<String>,
}

impl Compiler {
    pub fn new(nombre: impl Into<String>) -> Self {
        Self {
            chunk: Chunk::new(nombre),
            locals: Vec::new(),
            scope_depth: 0,
            duro_globals: std::collections::HashSet::new(),
        }
    }

    /// Consume el compilador y retorna el [`Chunk`] resultante.
    pub fn compile(mut self, stmts: &[Stmt]) -> Result<Chunk, CompileError> {
        for stmt in stmts {
            self.stmt(stmt)?;
        }
        self.chunk.emit_opcode(OpCode::RetornarNada, 0);
        Ok(self.chunk)
    }

    fn stmt(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Expresion(expr) => {
                self.expr(expr)?;
                // Las expresiones-statement descartan su valor.
                self.chunk.emit_opcode(OpCode::Pop, 0);
            }

            Stmt::DeclWea { nombre, valor, es_duro } => {
                self.expr(valor)?;
                if self.scope_depth == 0 {
                    self.define_global(nombre, *es_duro);
                } else {
                    self.define_local(nombre.clone(), *es_duro);
                }
            }

            Stmt::Cachai { cond, entonces, si_no } => {
                self.stmt_cachai(cond, entonces, si_no.as_deref())?;
            }

            Stmt::Mientras { cond, cuerpo } => {
                self.stmt_mientras(cond, cuerpo)?;
            }

            Stmt::Devolver { valor } => {
                self.expr(valor)?;
                self.chunk.emit_opcode(OpCode::Devolver, 0);
            }

            // Pendientes: requieren Value::Funcion + call frames en el VM.
            Stmt::DeclPega { .. } => todo!("funciones — próximo milestone"),

            // Pendientes: requieren iteradores en el VM.
            Stmt::Para { .. } => todo!("para — pendiente"),

            // Pendientes: requieren manejo de excepciones en el VM.
            Stmt::Ojo { .. } => todo!("ojo/cago — pendiente"),

            // Pendientes: requieren parchear saltos al inicio/fin del loop contenedor.
            Stmt::Cortala | Stmt::Sigue => todo!("cortala/sigue — pendiente"),
        }
        Ok(())
    }

    fn stmt_cachai(
        &mut self,
        cond: &Expr,
        entonces: &[Stmt],
        si_no: Option<&[Stmt]>,
    ) -> Result<(), CompileError> {
        self.expr(cond)?;

        let patch_falso = self.chunk.emit_jump(OpCode::SaltarSiFalso, 0);
        self.chunk.emit_opcode(OpCode::Pop, 0); // pop cond (rama verdadera)

        self.begin_scope();
        for s in entonces {
            self.stmt(s)?;
        }
        self.end_scope();

        // Siempre emitimos Saltar, incluso sin `si no`.
        // Garantiza que cada rama pop-ea la condición exactamente una vez:
        //   [cond] SaltarSiFalso→[A] Pop [entonces] Saltar→[B] [A]: Pop [si_no] [B]:
        let patch_fin = self.chunk.emit_jump(OpCode::Saltar, 0);
        self.chunk.patch_jump(patch_falso);
        self.chunk.emit_opcode(OpCode::Pop, 0); // pop cond (rama falsa)

        if let Some(ramas) = si_no {
            self.begin_scope();
            for s in ramas {
                self.stmt(s)?;
            }
            self.end_scope();
        }

        self.chunk.patch_jump(patch_fin);
        Ok(())
    }

    fn stmt_mientras(&mut self, cond: &Expr, cuerpo: &[Stmt]) -> Result<(), CompileError> {
        // Guardar la posición antes de la condición para el salto de vuelta.
        let loop_start = self.chunk.code.len();

        self.expr(cond)?;
        let patch_salir = self.chunk.emit_jump(OpCode::SaltarSiFalso, 0);
        self.chunk.emit_opcode(OpCode::Pop, 0);

        self.begin_scope();
        for s in cuerpo {
            self.stmt(s)?;
        }
        self.end_scope();

        // Volver al inicio para reevaluar la condición.
        self.chunk.emit_loop(loop_start, 0);

        // Aterriza aquí cuando la condición es falsa; pop la condición.
        self.chunk.patch_jump(patch_salir);
        self.chunk.emit_opcode(OpCode::Pop, 0);
        Ok(())
    }

    fn expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Numero(n) => {
                self.chunk.emit_constant(Value::Numero(*n), 0);
            }
            Expr::Texto(s) => {
                self.chunk.emit_constant(Value::from(s.as_str()), 0);
            }
            Expr::Booleano(true) => self.chunk.emit_opcode(OpCode::Verdad, 0),
            Expr::Booleano(false) => self.chunk.emit_opcode(OpCode::Falso, 0),
            Expr::Nada => self.chunk.emit_opcode(OpCode::Nada, 0),

            Expr::Ident(nombre, _span) => {
                self.expr_ident(nombre);
            }

            Expr::Asignacion { nombre, valor, .. } => {
                self.expr(valor)?;
                self.expr_asignacion(nombre)?;
            }

            Expr::Unario { op, expr, .. } => {
                self.expr(expr)?;
                match op {
                    OpUn::Neg => self.chunk.emit_opcode(OpCode::Neg, 0),
                    OpUn::No => self.chunk.emit_opcode(OpCode::No, 0),
                }
            }

            Expr::Binario { izq, op, der, .. } => {
                self.expr_binario(izq, op, der)?;
            }

            Expr::Llamada { callee, args, .. } => {
                self.expr_llamada(callee, args)?;
            }

            Expr::Lista(elementos, _) => {
                for el in elementos {
                    self.expr(el)?;
                }
                self.chunk.emit_opcode(OpCode::ConstruirLista, 0);
                self.chunk.emit_u16(elementos.len() as u16, 0);
            }

            Expr::Mapa(pares, _) => {
                for (k, v) in pares {
                    self.expr(k)?;
                    self.expr(v)?;
                }
                self.chunk.emit_opcode(OpCode::ConstruirMapa, 0);
                self.chunk.emit_u16(pares.len() as u16, 0);
            }

            Expr::Indice { objeto, indice, .. } => {
                self.expr(objeto)?;
                self.expr(indice)?;
                self.chunk.emit_opcode(OpCode::ObtenerIndice, 0);
            }
        }
        Ok(())
    }

    fn expr_ident(&mut self, nombre: &str) {
        if let Some(slot) = self.resolve_local(nombre) {
            self.chunk.emit_opcode(OpCode::ObtenerLocal, 0);
            self.chunk.emit_byte(slot, 0);
        } else {
            let idx = self.chunk.add_constant(Value::from(nombre));
            self.chunk.emit_opcode(OpCode::ObtenerGlobal, 0);
            self.chunk.emit_u16(idx, 0);
        }
    }

    fn expr_asignacion(&mut self, nombre: &str) -> Result<(), CompileError> {
        if let Some(slot) = self.resolve_local(nombre) {
            if self.locals[slot as usize].es_duro {
                return Err(CompileError::AsignacionConstante(nombre.to_string()));
            }
            self.chunk.emit_opcode(OpCode::AsignarLocal, 0);
            self.chunk.emit_byte(slot, 0);
        } else {
            if self.duro_globals.contains(nombre) {
                return Err(CompileError::AsignacionConstante(nombre.to_string()));
            }
            let idx = self.chunk.add_constant(Value::from(nombre));
            self.chunk.emit_opcode(OpCode::AsignarGlobal, 0);
            self.chunk.emit_u16(idx, 0);
        }
        Ok(())
    }


    fn expr_binario(&mut self, izq: &Expr, op: &OpBin, der: &Expr) -> Result<(), CompileError> {
        // `y` y `o` usan short-circuit: no evaluamos el lado derecho si no hace falta.
        match op {
            OpBin::Y => {
                self.expr(izq)?;
                // Si izq es falso, saltar al final y dejar izq como resultado.
                let patch = self.chunk.emit_jump(OpCode::SaltarSiFalso, 0);
                self.chunk.emit_opcode(OpCode::Pop, 0);
                self.expr(der)?;
                self.chunk.patch_jump(patch);
                return Ok(());
            }
            OpBin::O => {
                self.expr(izq)?;
                // Si izq es falso, saltar al lado derecho.
                // Si es verdadero, saltar sobre el lado derecho.
                let patch_falso = self.chunk.emit_jump(OpCode::SaltarSiFalso, 0);
                let patch_verdad = self.chunk.emit_jump(OpCode::Saltar, 0);
                self.chunk.patch_jump(patch_falso);
                self.chunk.emit_opcode(OpCode::Pop, 0);
                self.expr(der)?;
                self.chunk.patch_jump(patch_verdad);
                return Ok(());
            }
            _ => {}
        }

        self.expr(izq)?;
        self.expr(der)?;

        let opcode = match op {
            OpBin::Suma => OpCode::Suma,
            OpBin::Resta => OpCode::Resta,
            OpBin::Mul => OpCode::Mul,
            OpBin::Div => OpCode::Div,
            OpBin::Mod => OpCode::Mod,
            OpBin::Eq => OpCode::Eq,
            OpBin::Neq => OpCode::Neq,
            OpBin::Lt => OpCode::Lt,
            OpBin::Gt => OpCode::Gt,
            OpBin::Lte => OpCode::Lte,
            OpBin::Gte => OpCode::Gte,
            OpBin::Y | OpBin::O => unreachable!(),
        };
        self.chunk.emit_opcode(opcode, 0);
        Ok(())
    }

    fn expr_llamada(&mut self, callee: &Expr, args: &[Expr]) -> Result<(), CompileError> {
        // lorea/altiro son builtins con opcode propio; no necesitan call frame.
        // Lorea hace pop + print pero no deja valor: emitimos Nada explícitamente
        // para mantener la invariante: toda expresión deja exactamente un valor.
        if let Expr::Ident(nombre, _) = callee
            && (nombre == "lorea" || nombre == "altiro")
            && args.len() == 1
        {
            self.expr(&args[0])?;
            self.chunk.emit_opcode(OpCode::Lorea, 0);
            self.chunk.emit_opcode(OpCode::Nada, 0);
            return Ok(());
        }

        // Llamada general: callee en el stack, después los argumentos.
        // El VM buscará la función en stack[sp - 1 - n_args].
        self.expr(callee)?;
        for arg in args {
            self.expr(arg)?;
        }
        let n_args = u8::try_from(args.len()).map_err(|_| CompileError::DemasiadosArgumentos)?;
        self.chunk.emit_opcode(OpCode::Llamar, 0);
        self.chunk.emit_byte(n_args, 0);
        Ok(())
    }

    fn define_global(&mut self, nombre: &str, es_duro: bool) {
        if es_duro {
            self.duro_globals.insert(nombre.to_string());
        }
        let idx = self.chunk.add_constant(Value::from(nombre));
        self.chunk.emit_opcode(OpCode::DefinirGlobal, 0);
        self.chunk.emit_u16(idx, 0);
    }

    fn define_local(&mut self, nombre: String, es_duro: bool) {
        self.locals.push(Local {
            nombre,
            depth: self.scope_depth,
            es_duro,
        });
    }

    /// Busca de arriba (más reciente) hacia abajo para respetar shadowing.
    fn resolve_local(&self, nombre: &str) -> Option<u8> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .find(|(_, l)| l.nombre == nombre)
            .map(|(i, _)| i as u8)
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        // Pop todos los locales del scope que termina.
        while self.locals.last().is_some_and(|l| l.depth > self.scope_depth) {
            self.chunk.emit_opcode(OpCode::Pop, 0);
            self.locals.pop();
        }
    }
}

/// Atajo para compilar un programa completo desde el nivel raíz.
pub fn compilar(stmts: &[Stmt]) -> Result<Chunk, CompileError> {
    Compiler::new("<script>").compile(stmts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wn::{lexer::tokenizar, parser::parsear};

    fn compile_src(src: &str) -> Chunk {
        let tokens = tokenizar(src).unwrap();
        let stmts = parsear(tokens, src, "<test>").unwrap();
        compilar(&stmts).unwrap()
    }

    #[test]
    fn decl_global_simple() {
        let chunk = compile_src("wea x = 10 + 20");
        // Constante 10, Constante 20, Suma, DefinirGlobal "x", RetornarNada
        assert_eq!(OpCode::try_from(chunk.code[0]), Ok(OpCode::Constante));
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn lorea_builtin() {
        let chunk = compile_src("lorea(42)");
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn cachai_sin_si_no() {
        let chunk = compile_src("cachai (verdad) { lorea(1) }");
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn cachai_con_si_no() {
        let chunk = compile_src("cachai (falso) { lorea(1) } si no { lorea(2) }");
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn mientras_basico() {
        let chunk = compile_src("wea i = 0\nmientras (i < 3) { i = i + 1 }");
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn short_circuit_y() {
        let chunk = compile_src("verdad y falso");
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn locales_en_scope() {
        // Dentro del bloque, x es local slot 0; fuera no existe.
        let chunk = compile_src("cachai (verdad) { wea x = 5\nlorea(x) }");
        insta::assert_snapshot!(chunk.to_string());
    }
}
