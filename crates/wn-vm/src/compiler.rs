//! Compilador de AST a bytecode.
//!
//! Recorre el AST y emite instrucciones en un [`Chunk`] con metadata de span
//! por byte, para que el VM pueda reconstruir diagnósticos exactos.

use std::{collections::HashSet, rc::Rc, sync::Arc};

use wn::{
    ast::{Expr, OpBin, OpUn, Stmt},
    lexer::token::Span,
};
use wn_diagnostics::{SourceFile, WnDiagnostic};

use crate::{
    chunk::Chunk,
    opcode::OpCode,
    value::{ObjFunction, UpvalueDescriptor, Value},
};

#[derive(Debug)]
enum CompileErrorKind {
    AsignacionConstante(String),
    DemasiadosArgumentos,
    DemasiadosParametros(String),
    ControlFueraDeLoop(&'static str),
    DevolverFueraDeFuncion,
}

#[derive(Debug, Clone)]
struct Local {
    nombre: String,
    depth: u32,
    es_duro: bool,
    is_captured: bool,
    global_mirror: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionKind {
    Script,
    Function,
}

#[derive(Debug)]
struct FunctionContext {
    chunk: Chunk,
    locals: Vec<Local>,
    upvalues: Vec<UpvalueDescriptor>,
    scope_depth: u32,
    parent: Option<usize>,
    loop_stack: Vec<LoopContext>,
    hidden_counter: usize,
    kind: FunctionKind,
}

#[derive(Debug, Clone)]
struct LoopContext {
    continue_target: usize,
    break_patches: Vec<usize>,
    scope_depth: u32,
}

impl FunctionContext {
    fn new_script(nombre: impl Into<String>, source: Arc<SourceFile>) -> Self {
        Self {
            chunk: Chunk::new(nombre, source),
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
            parent: None,
            loop_stack: Vec::new(),
            hidden_counter: 0,
            kind: FunctionKind::Script,
        }
    }

    fn new_function(nombre: impl Into<String>, parent: usize, source: Arc<SourceFile>) -> Self {
        Self {
            chunk: Chunk::new(nombre, source),
            locals: vec![Local {
                nombre: String::new(),
                depth: 0,
                es_duro: false,
                is_captured: false,
                global_mirror: None,
            }],
            upvalues: Vec::new(),
            scope_depth: 1,
            parent: Some(parent),
            loop_stack: Vec::new(),
            hidden_counter: 0,
            kind: FunctionKind::Function,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum VariableRef {
    Local(u8),
    Upvalue(u8),
    Global,
}

pub struct Compiler {
    contexts: Vec<FunctionContext>,
    duro_globals: HashSet<String>,
    script_locals_enabled: bool,
}

impl Compiler {
    pub fn new(nombre: impl Into<String>, source: Arc<SourceFile>) -> Self {
        Self {
            contexts: vec![FunctionContext::new_script(nombre, source)],
            duro_globals: HashSet::new(),
            script_locals_enabled: true,
        }
    }

    pub fn compile(mut self, stmts: &[Stmt]) -> Result<Chunk, WnDiagnostic> {
        self.compile_impl(stmts, false)
    }

    pub fn compile_repl(mut self, stmts: &[Stmt]) -> Result<Chunk, WnDiagnostic> {
        self.script_locals_enabled = false;
        self.compile_impl(stmts, true)
    }

    fn compile_impl(&mut self, stmts: &[Stmt], repl: bool) -> Result<Chunk, WnDiagnostic> {
        for (idx, stmt) in stmts.iter().enumerate() {
            let es_ultima = idx + 1 == stmts.len();
            if repl
                && es_ultima
                && let Stmt::Expresion { expr, .. } = stmt
            {
                self.expr(expr)?;
                self.emit_opcode(OpCode::Devolver, expr.span().clone());
                return Ok(self.contexts.pop().expect("script context").chunk);
            }
            self.stmt(stmt)?;
        }
        self.emit_opcode(OpCode::RetornarNada, self.eof_span());
        Ok(self.contexts.pop().expect("script context").chunk)
    }

    fn stmt(&mut self, stmt: &Stmt) -> Result<(), WnDiagnostic> {
        match stmt {
            Stmt::Expresion { expr, .. } => {
                self.expr(expr)?;
                self.emit_opcode(OpCode::Pop, expr.span().clone());
            }
            Stmt::DeclWea {
                nombre,
                valor,
                es_duro,
                ..
            } => {
                self.expr(valor)?;
                if self.use_script_locals() {
                    let slot = self.define_script_local(nombre, *es_duro);
                    self.mirror_script_local_to_global(slot, stmt.span().clone());
                } else if self.scope_depth() == 0 {
                    self.define_global(nombre, *es_duro, stmt.span().clone());
                } else {
                    self.define_local(nombre.clone(), *es_duro);
                }
            }
            Stmt::DeclPega {
                nombre,
                params,
                cuerpo,
                ..
            } => self.stmt_decl_pega(nombre, params, cuerpo, stmt.span())?,
            Stmt::Cachai {
                cond,
                entonces,
                si_no,
                ..
            } => self.stmt_cachai(cond, entonces, si_no.as_deref())?,
            Stmt::Mientras { cond, cuerpo, .. } => self.stmt_mientras(cond, cuerpo)?,
            Stmt::Devolver { valor, span } => {
                if self.current_context().kind == FunctionKind::Script {
                    return Err(
                        self.compile_error(span.clone(), CompileErrorKind::DevolverFueraDeFuncion)
                    );
                }
                self.expr(valor)?;
                self.emit_opcode(OpCode::Devolver, span.clone());
            }
            Stmt::Para {
                var,
                iterable,
                cuerpo,
                ..
            } => self.stmt_para(var, iterable, cuerpo)?,
            Stmt::Ojo {
                cuerpo,
                error_var,
                manejo,
                ..
            } => self.stmt_ojo(cuerpo, error_var, manejo)?,
            Stmt::Cortala(span) => self.stmt_cortala(span)?,
            Stmt::Sigue(span) => self.stmt_sigue(span)?,
        }
        Ok(())
    }

    fn stmt_tail(&mut self, stmt: &Stmt) -> Result<(), WnDiagnostic> {
        match stmt {
            Stmt::Expresion { expr, .. } => self.expr(expr)?,
            Stmt::DeclWea {
                nombre,
                valor,
                es_duro,
                ..
            } => {
                self.expr(valor)?;
                if self.use_script_locals() {
                    let slot = self.define_script_local(nombre, *es_duro);
                    self.mirror_script_local_to_global(slot, stmt.span().clone());
                } else if self.scope_depth() == 0 {
                    self.define_global(nombre, *es_duro, stmt.span().clone());
                } else {
                    self.define_local(nombre.clone(), *es_duro);
                }
                self.emit_opcode(OpCode::Nada, stmt.span().clone());
            }
            Stmt::DeclPega {
                nombre,
                params,
                cuerpo,
                ..
            } => {
                self.stmt_decl_pega(nombre, params, cuerpo, stmt.span())?;
                self.emit_opcode(OpCode::Nada, stmt.span().clone());
            }
            Stmt::Cachai {
                cond,
                entonces,
                si_no,
                ..
            } => self.stmt_cachai_tail(cond, entonces, si_no.as_deref())?,
            Stmt::Mientras { cond, cuerpo, .. } => {
                self.stmt_mientras(cond, cuerpo)?;
                self.emit_opcode(OpCode::Nada, stmt.span().clone());
            }
            Stmt::Devolver { valor, span } => {
                if self.current_context().kind == FunctionKind::Script {
                    return Err(
                        self.compile_error(span.clone(), CompileErrorKind::DevolverFueraDeFuncion)
                    );
                }
                self.expr(valor)?;
                self.emit_opcode(OpCode::Devolver, span.clone());
            }
            Stmt::Para {
                var,
                iterable,
                cuerpo,
                ..
            } => {
                self.stmt_para(var, iterable, cuerpo)?;
                self.emit_opcode(OpCode::Nada, stmt.span().clone());
            }
            Stmt::Ojo {
                cuerpo,
                error_var,
                manejo,
                ..
            } => self.stmt_ojo_tail(cuerpo, error_var, manejo)?,
            Stmt::Cortala(span) => self.stmt_cortala(span)?,
            Stmt::Sigue(span) => self.stmt_sigue(span)?,
        }
        Ok(())
    }

    fn stmt_decl_pega(
        &mut self,
        nombre: &str,
        params: &[String],
        cuerpo: &[Stmt],
        span: &Span,
    ) -> Result<(), WnDiagnostic> {
        if self.use_script_locals() {
            self.emit_opcode(OpCode::Nada, span.clone());
            let slot = self.define_script_local(nombre, false);
            let closure = self.compile_function(nombre, params, cuerpo, span)?;
            self.emit_closure(closure, span.clone());
            self.emit_opcode(OpCode::AsignarLocal, span.clone());
            self.emit_byte(slot, span.clone());
            self.emit_opcode(OpCode::Pop, span.clone());
            self.mirror_script_local_to_global(slot, span.clone());
            return Ok(());
        }

        let closure = self.compile_function(nombre, params, cuerpo, span)?;
        self.emit_closure(closure, span.clone());
        if self.scope_depth() == 0 {
            self.define_global(nombre, false, span.clone());
        } else {
            self.define_local(nombre.to_string(), false);
        }
        Ok(())
    }

    fn stmt_cachai(
        &mut self,
        cond: &Expr,
        entonces: &[Stmt],
        si_no: Option<&[Stmt]>,
    ) -> Result<(), WnDiagnostic> {
        self.expr(cond)?;

        let patch_falso = self.emit_jump(OpCode::SaltarSiFalso, cond.span().clone());
        self.emit_opcode(OpCode::Pop, cond.span().clone());

        self.begin_scope();
        for stmt in entonces {
            self.stmt(stmt)?;
        }
        self.end_scope();

        let patch_fin = self.emit_jump(OpCode::Saltar, cond.span().clone());
        self.patch_jump(patch_falso);
        self.emit_opcode(OpCode::Pop, cond.span().clone());

        if let Some(rama_no) = si_no {
            self.begin_scope();
            for stmt in rama_no {
                self.stmt(stmt)?;
            }
            self.end_scope();
        }

        self.patch_jump(patch_fin);
        Ok(())
    }

    fn stmt_cachai_tail(
        &mut self,
        cond: &Expr,
        entonces: &[Stmt],
        si_no: Option<&[Stmt]>,
    ) -> Result<(), WnDiagnostic> {
        self.emit_opcode(OpCode::Nada, cond.span().clone());
        let result_slot = self.define_hidden_local("if_result");

        self.expr(cond)?;

        let patch_falso = self.emit_jump(OpCode::SaltarSiFalso, cond.span().clone());
        self.emit_opcode(OpCode::Pop, cond.span().clone());

        self.begin_scope();
        self.compile_stmts_tail(entonces)?;
        self.emit_opcode(OpCode::AsignarLocal, cond.span().clone());
        self.emit_byte(result_slot, cond.span().clone());
        self.emit_opcode(OpCode::Pop, cond.span().clone());
        self.end_scope();

        let patch_fin = self.emit_jump(OpCode::Saltar, cond.span().clone());
        self.patch_jump(patch_falso);
        self.emit_opcode(OpCode::Pop, cond.span().clone());

        if let Some(rama_no) = si_no {
            self.begin_scope();
            self.compile_stmts_tail(rama_no)?;
            self.emit_opcode(OpCode::AsignarLocal, cond.span().clone());
            self.emit_byte(result_slot, cond.span().clone());
            self.emit_opcode(OpCode::Pop, cond.span().clone());
            self.end_scope();
        } else {
            self.emit_opcode(OpCode::Nada, cond.span().clone());
            self.emit_opcode(OpCode::AsignarLocal, cond.span().clone());
            self.emit_byte(result_slot, cond.span().clone());
            self.emit_opcode(OpCode::Pop, cond.span().clone());
        }

        self.patch_jump(patch_fin);
        self.emit_opcode(OpCode::ObtenerLocal, cond.span().clone());
        self.emit_byte(result_slot, cond.span().clone());
        Ok(())
    }

    fn stmt_mientras(&mut self, cond: &Expr, cuerpo: &[Stmt]) -> Result<(), WnDiagnostic> {
        let loop_start = self.code_len();

        self.expr(cond)?;
        let patch_salir = self.emit_jump(OpCode::SaltarSiFalso, cond.span().clone());
        self.emit_opcode(OpCode::Pop, cond.span().clone());

        self.push_loop(loop_start, self.scope_depth());
        self.begin_scope();
        for stmt in cuerpo {
            self.stmt(stmt)?;
        }
        self.end_scope();

        self.emit_loop(loop_start, cond.span().clone());
        self.patch_jump(patch_salir);
        self.emit_opcode(OpCode::Pop, cond.span().clone());
        self.patch_breaks();
        Ok(())
    }

    fn stmt_para(
        &mut self,
        var: &str,
        iterable: &Expr,
        cuerpo: &[Stmt],
    ) -> Result<(), WnDiagnostic> {
        self.begin_scope();
        self.expr(iterable)?;
        self.emit_opcode(OpCode::IterInit, iterable.span().clone());
        let iter_name = self.hidden_name("iter");
        self.define_local(iter_name, false);
        let iter_slot = (self.current_context().locals.len() - 1) as u8;

        let loop_start = self.code_len();
        self.emit_opcode(OpCode::IterNext, iterable.span().clone());
        self.emit_byte(iter_slot, iterable.span().clone());
        let patch_salir = self.emit_jump(OpCode::SaltarSiFalso, iterable.span().clone());
        self.emit_opcode(OpCode::Pop, iterable.span().clone());

        self.push_loop(loop_start, self.scope_depth());
        self.begin_scope();
        self.define_local(var.to_string(), false);
        for stmt in cuerpo {
            self.stmt(stmt)?;
        }
        self.end_scope();

        self.emit_loop(loop_start, iterable.span().clone());
        self.patch_jump(patch_salir);
        self.emit_opcode(OpCode::Pop, iterable.span().clone());
        self.patch_breaks();
        self.end_scope();
        Ok(())
    }

    fn stmt_ojo(
        &mut self,
        cuerpo: &[Stmt],
        error_var: &str,
        manejo: &[Stmt],
    ) -> Result<(), WnDiagnostic> {
        let span = cuerpo
            .first()
            .map(|stmt| stmt.span().clone())
            .unwrap_or_else(|| self.eof_span());
        self.begin_scope();
        self.emit_opcode(OpCode::Nada, span.clone());
        let hidden_error = self.hidden_name("error");
        self.define_local(hidden_error, false);
        let hidden_slot = (self.current_context().locals.len() - 1) as u8;

        let patch_handler = self.emit_handler(hidden_slot, span.clone());

        self.begin_scope();
        for stmt in cuerpo {
            self.stmt(stmt)?;
        }
        self.end_scope();

        self.emit_opcode(OpCode::PopHandler, span.clone());
        let patch_end = self.emit_jump(OpCode::Saltar, span.clone());

        self.patch_handler(patch_handler);
        self.begin_scope();
        self.emit_opcode(OpCode::ObtenerLocal, span.clone());
        self.emit_byte(hidden_slot, span.clone());
        self.define_local(error_var.to_string(), false);
        for stmt in manejo {
            self.stmt(stmt)?;
        }
        self.end_scope();

        self.patch_jump(patch_end);
        self.end_scope();
        Ok(())
    }

    fn stmt_ojo_tail(
        &mut self,
        cuerpo: &[Stmt],
        error_var: &str,
        manejo: &[Stmt],
    ) -> Result<(), WnDiagnostic> {
        let span = cuerpo
            .first()
            .map(|stmt| stmt.span().clone())
            .unwrap_or_else(|| self.eof_span());
        self.begin_scope();
        self.emit_opcode(OpCode::Nada, span.clone());
        let hidden_error = self.hidden_name("error");
        self.define_local(hidden_error, false);
        let hidden_slot = (self.current_context().locals.len() - 1) as u8;

        self.emit_opcode(OpCode::Nada, span.clone());
        let result_slot = self.define_hidden_local("try_result");

        let patch_handler = self.emit_handler(hidden_slot, span.clone());

        self.begin_scope();
        self.compile_stmts_tail(cuerpo)?;
        self.emit_opcode(OpCode::AsignarLocal, span.clone());
        self.emit_byte(result_slot, span.clone());
        self.emit_opcode(OpCode::Pop, span.clone());
        self.end_scope();

        self.emit_opcode(OpCode::PopHandler, span.clone());
        let patch_end = self.emit_jump(OpCode::Saltar, span.clone());

        self.patch_handler(patch_handler);
        self.begin_scope();
        self.emit_opcode(OpCode::ObtenerLocal, span.clone());
        self.emit_byte(hidden_slot, span.clone());
        self.define_local(error_var.to_string(), false);
        self.compile_stmts_tail(manejo)?;
        self.emit_opcode(OpCode::AsignarLocal, span.clone());
        self.emit_byte(result_slot, span.clone());
        self.emit_opcode(OpCode::Pop, span.clone());
        self.end_scope();

        self.patch_jump(patch_end);
        self.emit_opcode(OpCode::ObtenerLocal, span.clone());
        self.emit_byte(result_slot, span.clone());
        self.end_scope();
        Ok(())
    }

    fn stmt_cortala(&mut self, span: &Span) -> Result<(), WnDiagnostic> {
        let loop_ctx = self
            .current_context()
            .loop_stack
            .last()
            .cloned()
            .ok_or_else(|| {
                self.compile_error(
                    span.clone(),
                    CompileErrorKind::ControlFueraDeLoop("cortala"),
                )
            })?;
        self.emit_unwind_to_depth(loop_ctx.scope_depth, span);
        let patch = self.emit_jump(OpCode::Saltar, span.clone());
        self.current_context_mut()
            .loop_stack
            .last_mut()
            .expect("loop activo")
            .break_patches
            .push(patch);
        Ok(())
    }

    fn stmt_sigue(&mut self, span: &Span) -> Result<(), WnDiagnostic> {
        let loop_ctx = self
            .current_context()
            .loop_stack
            .last()
            .cloned()
            .ok_or_else(|| {
                self.compile_error(span.clone(), CompileErrorKind::ControlFueraDeLoop("sigue"))
            })?;
        self.emit_unwind_to_depth(loop_ctx.scope_depth, span);
        self.emit_loop(loop_ctx.continue_target, span.clone());
        Ok(())
    }

    fn expr(&mut self, expr: &Expr) -> Result<(), WnDiagnostic> {
        match expr {
            Expr::Numero(n, span) => self.emit_constant(Value::Numero(*n), span.clone()),
            Expr::Texto(s, span) => self.emit_constant(Value::from(s.as_str()), span.clone()),
            Expr::Booleano(true, span) => self.emit_opcode(OpCode::Verdad, span.clone()),
            Expr::Booleano(false, span) => self.emit_opcode(OpCode::Falso, span.clone()),
            Expr::Nada(span) => self.emit_opcode(OpCode::Nada, span.clone()),
            Expr::Ident(nombre, span) => self.expr_ident(nombre, span.clone()),
            Expr::Asignacion {
                nombre,
                valor,
                span,
            } => {
                self.expr(valor)?;
                self.expr_asignacion(nombre, span)?;
            }
            Expr::Unario { op, expr, span } => {
                self.expr(expr)?;
                match op {
                    OpUn::Neg => self.emit_opcode(OpCode::Neg, span.clone()),
                    OpUn::No => self.emit_opcode(OpCode::No, span.clone()),
                }
            }
            Expr::Binario { izq, op, der, span } => self.expr_binario(izq, op, der, span)?,
            Expr::Llamada { callee, args, span } => self.expr_llamada(callee, args, span)?,
            Expr::Lista(elementos, span) => {
                for elemento in elementos {
                    self.expr(elemento)?;
                }
                self.emit_opcode(OpCode::ConstruirLista, span.clone());
                self.emit_u16(elementos.len() as u16, span.clone());
            }
            Expr::Mapa(pares, span) => {
                for (clave, valor) in pares {
                    self.expr(clave)?;
                    self.expr(valor)?;
                }
                self.emit_opcode(OpCode::ConstruirMapa, span.clone());
                self.emit_u16(pares.len() as u16, span.clone());
            }
            Expr::Indice {
                objeto,
                indice,
                span,
            } => {
                self.expr(objeto)?;
                self.expr(indice)?;
                self.emit_opcode(OpCode::ObtenerIndice, span.clone());
            }
            Expr::AsignacionIndice { objeto, indice, valor, span } => {
                self.expr(objeto)?;
                self.expr(indice)?;
                self.expr(valor)?;
                self.emit_opcode(OpCode::AsignarIndice, span.clone());
            }
        }
        Ok(())
    }

    fn expr_ident(&mut self, nombre: &str, span: Span) {
        match self.resolve_variable(nombre) {
            VariableRef::Local(slot) => {
                self.emit_opcode(OpCode::ObtenerLocal, span.clone());
                self.emit_byte(slot, span);
            }
            VariableRef::Upvalue(slot) => {
                self.emit_opcode(OpCode::ObtenerUpvalue, span.clone());
                self.emit_byte(slot, span);
            }
            VariableRef::Global => {
                let idx = self.add_constant(Value::from(nombre));
                self.emit_opcode(OpCode::ObtenerGlobal, span.clone());
                self.emit_u16(idx, span);
            }
        }
    }

    fn expr_asignacion(&mut self, nombre: &str, span: &Span) -> Result<(), WnDiagnostic> {
        match self.resolve_variable(nombre) {
            VariableRef::Local(slot) => {
                if self.current_context().locals[slot as usize].es_duro {
                    return Err(self.compile_error(
                        span.clone(),
                        CompileErrorKind::AsignacionConstante(nombre.to_string()),
                    ));
                }
                self.emit_opcode(OpCode::AsignarLocal, span.clone());
                self.emit_byte(slot, span.clone());
                if let Some(idx) = self.current_context().locals[slot as usize].global_mirror {
                    self.emit_opcode(OpCode::AsignarGlobal, span.clone());
                    self.emit_u16(idx, span.clone());
                }
            }
            VariableRef::Upvalue(slot) => {
                self.emit_opcode(OpCode::AsignarUpvalue, span.clone());
                self.emit_byte(slot, span.clone());
            }
            VariableRef::Global => {
                if self.duro_globals.contains(nombre) {
                    return Err(self.compile_error(
                        span.clone(),
                        CompileErrorKind::AsignacionConstante(nombre.to_string()),
                    ));
                }
                let idx = self.add_constant(Value::from(nombre));
                self.emit_opcode(OpCode::AsignarGlobal, span.clone());
                self.emit_u16(idx, span.clone());
            }
        }
        Ok(())
    }

    fn expr_binario(
        &mut self,
        izq: &Expr,
        op: &OpBin,
        der: &Expr,
        span: &Span,
    ) -> Result<(), WnDiagnostic> {
        match op {
            OpBin::Y => {
                self.expr(izq)?;
                let patch = self.emit_jump(OpCode::SaltarSiFalso, span.clone());
                self.emit_opcode(OpCode::Pop, span.clone());
                self.expr(der)?;
                self.patch_jump(patch);
                return Ok(());
            }
            OpBin::O => {
                self.expr(izq)?;
                let patch_falso = self.emit_jump(OpCode::SaltarSiFalso, span.clone());
                let patch_verdad = self.emit_jump(OpCode::Saltar, span.clone());
                self.patch_jump(patch_falso);
                self.emit_opcode(OpCode::Pop, span.clone());
                self.expr(der)?;
                self.patch_jump(patch_verdad);
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
        self.emit_opcode(opcode, span.clone());
        Ok(())
    }

    fn expr_llamada(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        span: &Span,
    ) -> Result<(), WnDiagnostic> {
        self.expr(callee)?;
        for arg in args {
            self.expr(arg)?;
        }
        let n_args = u8::try_from(args.len()).map_err(|_| {
            self.compile_error(span.clone(), CompileErrorKind::DemasiadosArgumentos)
        })?;
        self.emit_opcode(OpCode::Llamar, span.clone());
        self.emit_byte(n_args, span.clone());
        Ok(())
    }

    fn compile_function(
        &mut self,
        nombre: &str,
        params: &[String],
        cuerpo: &[Stmt],
        span: &Span,
    ) -> Result<Rc<ObjFunction>, WnDiagnostic> {
        if params.len() > u8::MAX as usize {
            return Err(self.compile_error(
                span.clone(),
                CompileErrorKind::DemasiadosParametros(nombre.to_string()),
            ));
        }

        let parent = self.contexts.len() - 1;
        let source = self.current_context().chunk.source.clone();
        self.contexts.push(FunctionContext::new_function(
            nombre.to_string(),
            parent,
            source,
        ));

        for param in params {
            self.define_local(param.clone(), false);
        }

        self.compile_stmts_tail(cuerpo)?;
        self.emit_opcode(OpCode::Devolver, span.clone());

        let ctx = self.contexts.pop().expect("function context");
        Ok(Rc::new(ObjFunction {
            aridad: params.len(),
            nombre: nombre.to_string(),
            upvalues: ctx.upvalues,
            chunk: ctx.chunk,
        }))
    }

    fn emit_closure(&mut self, funcion: Rc<ObjFunction>, span: Span) {
        let idx = self.add_constant(Value::Funcion(funcion.clone()));
        self.emit_opcode(OpCode::Closure, span.clone());
        self.emit_u16(idx, span.clone());
        for descriptor in &funcion.upvalues {
            self.emit_byte(u8::from(descriptor.is_local), span.clone());
            self.emit_byte(descriptor.index, span.clone());
        }
    }

    fn define_global(&mut self, nombre: &str, es_duro: bool, span: Span) {
        if es_duro {
            self.duro_globals.insert(nombre.to_string());
        }
        let idx = self.add_constant(Value::from(nombre));
        self.emit_opcode(OpCode::DefinirGlobal, span.clone());
        self.emit_u16(idx, span);
    }

    fn define_local(&mut self, nombre: String, es_duro: bool) {
        let depth = self.scope_depth();
        self.current_context_mut().locals.push(Local {
            nombre,
            depth,
            es_duro,
            is_captured: false,
            global_mirror: None,
        });
    }

    fn define_script_local(&mut self, nombre: &str, es_duro: bool) -> u8 {
        if es_duro {
            self.duro_globals.insert(nombre.to_string());
        }
        let idx = self.add_constant(Value::from(nombre));
        let depth = self.scope_depth();
        self.current_context_mut().locals.push(Local {
            nombre: nombre.to_string(),
            depth,
            es_duro,
            is_captured: false,
            global_mirror: Some(idx),
        });
        (self.current_context().locals.len() - 1) as u8
    }

    fn resolve_variable(&mut self, nombre: &str) -> VariableRef {
        let current_idx = self.contexts.len() - 1;
        if let Some(slot) = self.resolve_local(current_idx, nombre) {
            return VariableRef::Local(slot);
        }
        if let Some(slot) = self.resolve_upvalue(current_idx, nombre) {
            return VariableRef::Upvalue(slot);
        }
        VariableRef::Global
    }

    fn resolve_local(&self, ctx_idx: usize, nombre: &str) -> Option<u8> {
        self.contexts[ctx_idx]
            .locals
            .iter()
            .enumerate()
            .rev()
            .find(|(_, local)| local.nombre == nombre)
            .map(|(i, _)| i as u8)
    }

    fn resolve_upvalue(&mut self, ctx_idx: usize, nombre: &str) -> Option<u8> {
        let parent_idx = self.contexts[ctx_idx].parent?;
        if let Some(local_slot) = self.resolve_local(parent_idx, nombre) {
            self.contexts[parent_idx].locals[local_slot as usize].is_captured = true;
            return Some(self.add_upvalue(
                ctx_idx,
                UpvalueDescriptor {
                    index: local_slot,
                    is_local: true,
                },
            ));
        }

        let parent_upvalue = self.resolve_upvalue(parent_idx, nombre)?;
        Some(self.add_upvalue(
            ctx_idx,
            UpvalueDescriptor {
                index: parent_upvalue,
                is_local: false,
            },
        ))
    }

    fn add_upvalue(&mut self, ctx_idx: usize, descriptor: UpvalueDescriptor) -> u8 {
        if let Some((idx, _)) = self.contexts[ctx_idx]
            .upvalues
            .iter()
            .enumerate()
            .find(|(_, up)| **up == descriptor)
        {
            return idx as u8;
        }

        self.contexts[ctx_idx].upvalues.push(descriptor);
        (self.contexts[ctx_idx].upvalues.len() - 1) as u8
    }

    fn begin_scope(&mut self) {
        self.current_context_mut().scope_depth += 1;
    }

    fn end_scope(&mut self) {
        let new_depth = self.scope_depth().saturating_sub(1);
        self.current_context_mut().scope_depth = new_depth;

        while self
            .current_context()
            .locals
            .last()
            .is_some_and(|local| local.depth > new_depth)
        {
            let local = self.current_context_mut().locals.pop().expect("local");
            let span = self.eof_span();
            if local.is_captured {
                self.emit_opcode(OpCode::CerrarUpvalue, span);
            } else {
                self.emit_opcode(OpCode::Pop, span);
            }
        }
    }

    fn emit_unwind_to_depth(&mut self, target_depth: u32, span: &Span) {
        let cleanup = self
            .current_context()
            .locals
            .iter()
            .rev()
            .take_while(|local| local.depth > target_depth)
            .map(|local| local.is_captured)
            .collect::<Vec<_>>();

        for is_captured in cleanup {
            if is_captured {
                self.emit_opcode(OpCode::CerrarUpvalue, span.clone());
            } else {
                self.emit_opcode(OpCode::Pop, span.clone());
            }
        }
    }

    fn hidden_name(&mut self, base: &str) -> String {
        let counter = self.current_context().hidden_counter;
        self.current_context_mut().hidden_counter += 1;
        format!("__wn_{base}_{counter}")
    }

    fn define_hidden_local(&mut self, base: &str) -> u8 {
        let hidden_name = self.hidden_name(base);
        self.define_local(hidden_name, false);
        (self.current_context().locals.len() - 1) as u8
    }

    fn mirror_script_local_to_global(&mut self, slot: u8, span: Span) {
        let idx = self.current_context().locals[slot as usize]
            .global_mirror
            .expect("script local sin mirror global");
        self.emit_opcode(OpCode::ObtenerLocal, span.clone());
        self.emit_byte(slot, span.clone());
        self.emit_opcode(OpCode::DefinirGlobal, span.clone());
        self.emit_u16(idx, span);
    }

    fn use_script_locals(&self) -> bool {
        self.script_locals_enabled
            && self.current_context().kind == FunctionKind::Script
            && self.scope_depth() == 0
    }

    fn compile_stmts_tail(&mut self, stmts: &[Stmt]) -> Result<(), WnDiagnostic> {
        if let Some((last, init)) = stmts.split_last() {
            for stmt in init {
                self.stmt(stmt)?;
            }
            self.stmt_tail(last)?;
        } else {
            self.emit_opcode(OpCode::Nada, self.eof_span());
        }
        Ok(())
    }

    fn push_loop(&mut self, continue_target: usize, scope_depth: u32) {
        self.current_context_mut().loop_stack.push(LoopContext {
            continue_target,
            break_patches: Vec::new(),
            scope_depth,
        });
    }

    fn patch_breaks(&mut self) {
        let loop_ctx = self
            .current_context_mut()
            .loop_stack
            .pop()
            .expect("loop activo");
        for patch in loop_ctx.break_patches {
            self.patch_jump(patch);
        }
    }

    fn current_context(&self) -> &FunctionContext {
        self.contexts.last().expect("compiler context")
    }

    fn current_context_mut(&mut self) -> &mut FunctionContext {
        self.contexts.last_mut().expect("compiler context")
    }

    fn scope_depth(&self) -> u32 {
        self.current_context().scope_depth
    }

    fn code_len(&self) -> usize {
        self.current_context().chunk.code.len()
    }

    fn eof_span(&self) -> Span {
        let len = self.current_context().chunk.source.src().len();
        Span::new(len, len)
    }

    fn add_constant(&mut self, value: Value) -> u16 {
        self.current_context_mut().chunk.add_constant(value)
    }

    fn emit_constant(&mut self, value: Value, span: Span) {
        self.current_context_mut().chunk.emit_constant(value, span);
    }

    fn emit_byte(&mut self, byte: u8, span: Span) {
        self.current_context_mut().chunk.emit_byte(byte, span);
    }

    fn emit_u16(&mut self, value: u16, span: Span) {
        self.current_context_mut().chunk.emit_u16(value, span);
    }

    fn emit_opcode(&mut self, op: OpCode, span: Span) {
        self.current_context_mut().chunk.emit_opcode(op, span);
    }

    fn emit_jump(&mut self, op: OpCode, span: Span) -> usize {
        self.current_context_mut().chunk.emit_jump(op, span)
    }

    fn emit_handler(&mut self, error_slot: u8, span: Span) -> usize {
        self.emit_opcode(OpCode::PushHandler, span.clone());
        self.emit_byte(0xFF, span.clone());
        self.emit_byte(0xFF, span.clone());
        self.emit_byte(error_slot, span);
        self.code_len() - 3
    }

    fn patch_jump(&mut self, patch_offset: usize) {
        self.current_context_mut().chunk.patch_jump(patch_offset);
    }

    fn patch_handler(&mut self, patch_offset: usize) {
        let jump = self.code_len() - patch_offset - 3;
        self.current_context_mut().chunk.code[patch_offset] = (jump >> 8) as u8;
        self.current_context_mut().chunk.code[patch_offset + 1] = (jump & 0xFF) as u8;
    }

    fn emit_loop(&mut self, loop_start: usize, span: Span) {
        self.current_context_mut().chunk.emit_loop(loop_start, span);
    }

    fn compile_error(&self, span: Span, kind: CompileErrorKind) -> WnDiagnostic {
        let source = self.current_context().chunk.source.clone();
        let span = source.span(span.start, span.end);
        match kind {
            CompileErrorKind::AsignacionConstante(nombre) => {
                WnDiagnostic::constante_inmutable(&source, span, nombre)
            }
            CompileErrorKind::DemasiadosArgumentos => WnDiagnostic::compilacion(
                &source,
                span,
                "Demasiados argumentos en la llamada (máximo 255).",
            ),
            CompileErrorKind::DemasiadosParametros(nombre) => WnDiagnostic::compilacion(
                &source,
                span,
                format!("Demasiados parámetros en la pega '{nombre}' (máximo 255)."),
            ),
            CompileErrorKind::ControlFueraDeLoop(control) => WnDiagnostic::compilacion(
                &source,
                span,
                format!("'{control}' solo tiene sentido dentro de un bucle, compare."),
            ),
            CompileErrorKind::DevolverFueraDeFuncion => WnDiagnostic::compilacion(
                &source,
                span,
                "'devolver' solo puede usarse dentro de una pega papito.",
            ),
        }
    }
}

pub fn compilar(stmts: &[Stmt], source: Arc<SourceFile>) -> Result<Chunk, WnDiagnostic> {
    Compiler::new("<script>", source).compile(stmts)
}

pub fn compilar_repl(stmts: &[Stmt], source: Arc<SourceFile>) -> Result<Chunk, WnDiagnostic> {
    Compiler::new("<repl>", source).compile_repl(stmts)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use wn::{lexer::tokenizar, parser::parsear};

    fn compile_src(src: &str) -> Chunk {
        let tokens = tokenizar(src).unwrap();
        let stmts = parsear(tokens, src, "<test>").unwrap();
        let source = Arc::new(SourceFile::new("<test>", src));
        compilar(&stmts, source).unwrap()
    }

    #[test]
    fn decl_global_simple() {
        let chunk = compile_src("wea x = 10 + 20");
        assert_eq!(OpCode::try_from(chunk.code[0]), Ok(OpCode::Constante));
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn llamada_nativa_general() {
        let chunk = compile_src("lorea(42)");
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn cachai_sin_si_no() {
        let chunk = compile_src("cachai (verdad) { lorea(1) }");
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn mientras_basico() {
        let chunk = compile_src("wea i = 0\nmientras (i < 3) { i = i + 1 }");
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn compila_pega_simple() {
        let chunk = compile_src("pega sumar(a, b) { devolver a + b }");
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn compila_closure_con_captura() {
        let chunk =
            compile_src("pega afuera(x) { pega adentro() { devolver x }\n devolver adentro }");
        insta::assert_snapshot!(chunk.to_string());
    }

    #[test]
    fn devolver_fuera_de_funcion_reporta_span() {
        let src = "devolver 1";
        let tokens = tokenizar(src).unwrap();
        let stmts = parsear(tokens, src, "<test>").unwrap();
        let source = Arc::new(SourceFile::new("<test>", src));
        let err = compilar(&stmts, source).unwrap_err();

        match err {
            WnDiagnostic::Compilacion { span, .. } => {
                assert_eq!(span.offset(), 0usize);
                assert_eq!(span.len(), 10);
            }
            other => panic!("diagnóstico inesperado: {other:?}"),
        }
    }
}
