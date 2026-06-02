//! Un bloque de bytecode compilado.

use std::{fmt, sync::Arc};

use crate::{opcode::OpCode, value::Value};
use wn::lexer::token::Span;
use wn_diagnostics::SourceFile;

/// Unidad de compilación: instrucciones + constantes + info de debug.
///
/// Los campos son `pub` deliberadamente: el `Compiler` escribe en ellos
/// directamente y el `VM` los lee directamente. No hay invariante que ocultar.
#[derive(Clone)]
pub struct Chunk {
    /// Bytecode: secuencia plana de opcodes y sus operandos.
    pub code: Vec<u8>,
    /// Pool de constantes referenciadas por instrucciones `CONSTANTE <u16>`.
    pub constants: Vec<Value>,
    /// Span fuente asociado a cada byte en `code`.
    pub spans: Vec<Span>,
    /// Número de línea fuente para cada byte en `code` (array paralelo).
    /// Permite que el VM reporte "error en línea 42" aunque esté en bytecode.
    pub lines: Vec<u32>,
    /// Nombre descriptivo para output de debug (ej: `"<script>"`, `"sumar"`).
    pub name: String,
    /// Archivo fuente dueño del chunk.
    pub source: Arc<SourceFile>,
}

impl Chunk {
    pub fn new(name: impl Into<String>, source: Arc<SourceFile>) -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            spans: Vec::new(),
            lines: Vec::new(),
            name: name.into(),
            source,
        }
    }

    /// Escribe un byte crudo (opcode u operando) y registra su línea fuente.
    pub fn emit_byte(&mut self, byte: u8, span: Span) {
        self.code.push(byte);
        self.lines.push(self.source.line_for_offset(span.start));
        self.spans.push(span);
    }

    /// Escribe un opcode (azúcar sobre `emit_byte`).
    pub fn emit_opcode(&mut self, op: OpCode, span: Span) {
        self.emit_byte(op as u8, span);
    }

    /// Escribe un `u16` en big-endian (2 bytes, mismo `line` para ambos).
    pub fn emit_u16(&mut self, value: u16, span: Span) {
        self.emit_byte((value >> 8) as u8, span.clone());
        self.emit_byte((value & 0xFF) as u8, span);
    }

    /// Agrega `value` al pool y retorna su índice como `u16`.
    ///
    /// # Panics
    /// Si el pool supera los 65535 valores.
    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        let idx = self.constants.len() - 1;
        assert!(
            idx <= u16::MAX as usize,
            "límite de constantes por chunk superado (máx {})",
            u16::MAX
        );
        idx as u16
    }

    /// Emite `OpConstante` + índice u16. Atajo para el compilador.
    pub fn emit_constant(&mut self, value: Value, span: Span) {
        let idx = self.add_constant(value);
        self.emit_opcode(OpCode::Constante, span.clone());
        self.emit_u16(idx, span);
    }

    /// Emite `op` con operando placeholder `0xFFFF`.
    /// Retorna la posición del placeholder para parcharlo luego con
    /// [`patch_jump`](Self::patch_jump).
    ///
    /// Patrón de uso:
    /// ```ignore
    /// let patch = chunk.emit_jump(OpCode::SaltarSiFalso, line);
    /// // ... compilar cuerpo del `cachai` ...
    /// chunk.patch_jump(patch);
    /// ```
    pub fn emit_jump(&mut self, op: OpCode, span: Span) -> usize {
        self.emit_opcode(op, span.clone());
        self.emit_byte(0xFF, span.clone()); // placeholder high byte
        self.emit_byte(0xFF, span); // placeholder low byte
        self.code.len() - 2 // posición del placeholder
    }

    /// Rellena el destino de un salto emitido con `emit_jump`.
    /// El offset se calcula desde el byte *después* del operando.
    ///
    /// # Panics
    /// Si la distancia supera `u16::MAX`.
    pub fn patch_jump(&mut self, patch_offset: usize) {
        let jump = self.code.len() - patch_offset - 2;
        assert!(
            jump <= u16::MAX as usize,
            "salto demasiado largo: {} bytes",
            jump
        );
        self.code[patch_offset] = (jump >> 8) as u8;
        self.code[patch_offset + 1] = (jump & 0xFF) as u8;
    }

    /// Emite `OpLoop` con offset hacia atrás a `loop_start`.
    /// Llamar al final del cuerpo de un `mientras`.
    pub fn emit_loop(&mut self, loop_start: usize, span: Span) {
        self.emit_opcode(OpCode::Loop, span.clone());
        // +2: los 2 bytes del operando que escribiremos ahora
        let offset = self.code.len() - loop_start + 2;
        assert!(
            offset <= u16::MAX as usize,
            "cuerpo de loop demasiado largo: {} bytes",
            offset
        );
        self.emit_u16(offset as u16, span);
    }

    /// Imprime el chunk desensamblado a stdout. Útil durante el desarrollo del compilador.
    pub fn disassemble(&self) {
        print!("{self}");
    }

    /// Desensambla la instrucción en `offset` y la imprime a stdout.
    /// Retorna el offset de la siguiente instrucción.
    pub fn disassemble_at(&self, offset: usize) -> usize {
        // Usamos un String como buffer intermedio porque fmt_instruction
        // requiere fmt::Write, no io::Write (que es lo que tiene stdout).
        let mut buf = String::new();
        match self.fmt_instruction(&mut buf, offset) {
            Ok(next) => {
                print!("{buf}");
                next
            }
            Err(_) => offset + 1,
        }
    }

    /// Núcleo del desensamblador. Escribe una instrucción en `f` y retorna
    /// el offset de la siguiente.
    fn fmt_instruction(&self, f: &mut impl fmt::Write, offset: usize) -> Result<usize, fmt::Error> {
        write!(f, "{offset:04x}  ")?;

        // Número de línea: solo muestra cuando cambia (compresión RLE)
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            write!(f, "   |  ")?;
        } else {
            write!(f, "{:4}  ", self.lines[offset])?;
        }

        let byte = self.code[offset];
        match OpCode::try_from(byte) {
            Err(b) => {
                writeln!(f, "OPCODE DESCONOCIDO ({b:#04x})")?;
                Ok(offset + 1)
            }
            Ok(op) => match op {
                OpCode::Constante => self.fmt_constante(f, offset),

                OpCode::DefinirGlobal | OpCode::ObtenerGlobal | OpCode::AsignarGlobal => {
                    self.fmt_u16_con_nombre(f, op, offset)
                }

                OpCode::ObtenerLocal | OpCode::AsignarLocal => self.fmt_u8_arg(f, op, offset),

                OpCode::Saltar | OpCode::SaltarSiFalso => self.fmt_salto_adelante(f, op, offset),

                OpCode::Loop => self.fmt_salto_atras(f, offset),

                OpCode::PushHandler => self.fmt_handler(f, offset),

                OpCode::Closure => self.fmt_closure(f, offset),

                OpCode::Llamar
                | OpCode::ObtenerUpvalue
                | OpCode::AsignarUpvalue
                | OpCode::IterNext => self.fmt_u8_arg(f, op, offset),

                OpCode::ConstruirLista | OpCode::ConstruirMapa => self.fmt_u16_arg(f, op, offset),

                // Sin operandos: solo el nombre del opcode
                _ => {
                    writeln!(f, "{op:?}")?;
                    Ok(offset + 1)
                }
            },
        }
    }

    fn fmt_constante(&self, f: &mut impl fmt::Write, offset: usize) -> Result<usize, fmt::Error> {
        let idx = self.read_u16(offset + 1) as usize;
        writeln!(f, "{:<20} {idx:>5}  '{}'", "Constante", self.constants[idx])?;
        Ok(offset + 3)
    }

    fn fmt_u16_con_nombre(
        &self,
        f: &mut impl fmt::Write,
        op: OpCode,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        let idx = self.read_u16(offset + 1) as usize;
        writeln!(
            f,
            "{:<20} {idx:>5}  '{}'",
            format!("{op:?}"),
            self.constants[idx]
        )?;
        Ok(offset + 3)
    }

    fn fmt_u8_arg(
        &self,
        f: &mut impl fmt::Write,
        op: OpCode,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        let arg = self.code[offset + 1];
        writeln!(f, "{:<20} {arg:>5}", format!("{op:?}"))?;
        Ok(offset + 2)
    }

    fn fmt_u16_arg(
        &self,
        f: &mut impl fmt::Write,
        op: OpCode,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        let arg = self.read_u16(offset + 1);
        writeln!(f, "{:<20} {arg:>5}", format!("{op:?}"))?;
        Ok(offset + 3)
    }

    fn fmt_salto_adelante(
        &self,
        f: &mut impl fmt::Write,
        op: OpCode,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        let jump = self.read_u16(offset + 1) as usize;
        let dest = offset + 3 + jump;
        writeln!(f, "{:<20} {offset:>5} → {dest:04x}", format!("{op:?}"))?;
        Ok(offset + 3)
    }

    fn fmt_salto_atras(&self, f: &mut impl fmt::Write, offset: usize) -> Result<usize, fmt::Error> {
        let jump = self.read_u16(offset + 1) as usize;
        let dest = offset + 3 - jump;
        writeln!(f, "{:<20} {offset:>5} → {dest:04x}", "Loop")?;
        Ok(offset + 3)
    }

    fn fmt_closure(&self, f: &mut impl fmt::Write, offset: usize) -> Result<usize, fmt::Error> {
        let idx = self.read_u16(offset + 1) as usize;
        writeln!(f, "{:<20} {idx:>5}  '{}'", "Closure", self.constants[idx])?;

        let funcion = match &self.constants[idx] {
            Value::Funcion(funcion) => funcion,
            _ => panic!("constante de closure no es función"),
        };

        let mut next = offset + 3;
        for descriptor in &funcion.upvalues {
            writeln!(
                f,
                "{:04x}     |  {:<20} {}",
                next,
                "captura",
                if descriptor.is_local {
                    format!("local {}", descriptor.index)
                } else {
                    format!("upvalue {}", descriptor.index)
                }
            )?;
            next += 2;
        }

        Ok(next)
    }

    fn fmt_handler(&self, f: &mut impl fmt::Write, offset: usize) -> Result<usize, fmt::Error> {
        let jump = self.read_u16(offset + 1) as usize;
        let slot = self.code[offset + 3];
        writeln!(
            f,
            "{:<20} {offset:>5} → {:04x}  slot {slot}",
            "PushHandler",
            offset + 4 + jump
        )?;
        Ok(offset + 4)
    }

    /// Lee un `u16` big-endian desde `code[offset..]`.
    #[inline]
    fn read_u16(&self, offset: usize) -> u16 {
        ((self.code[offset] as u16) << 8) | (self.code[offset + 1] as u16)
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "══ {} ══", self.name)?;
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.fmt_instruction(f, offset)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chunk(\"{}\" {} bytes)", self.name, self.code.len())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use wn_diagnostics::SourceFile;

    fn test_source() -> Arc<SourceFile> {
        Arc::new(SourceFile::new("<test>", "wea x = 1"))
    }

    #[test]
    fn chunk_constante_simple() {
        let mut chunk = Chunk::new("<test>", test_source());
        let span = Span::new(0, 1);
        chunk.emit_constant(Value::Numero(1.0), span.clone());
        chunk.emit_constant(Value::Numero(2.0), span.clone());
        chunk.emit_opcode(OpCode::Suma, span.clone());
        chunk.emit_opcode(OpCode::RetornarNada, span);

        assert_eq!(chunk.code.len(), 8); // 3 bytes × 2 constantes + Suma + RetornarNada
        assert_eq!(chunk.constants.len(), 2);
        assert_eq!(chunk.constants[0], Value::Numero(1.0));
        assert_eq!(chunk.constants[1], Value::Numero(2.0));
        assert_eq!(OpCode::try_from(chunk.code[0]), Ok(OpCode::Constante));
    }

    #[test]
    fn patch_jump_correcto() {
        let mut chunk = Chunk::new("<test>", test_source());
        let span = Span::new(0, 1);
        let patch = chunk.emit_jump(OpCode::SaltarSiFalso, span.clone());
        chunk.emit_opcode(OpCode::Lorea, span);
        chunk.patch_jump(patch);

        let high = chunk.code[patch] as u16;
        let low = chunk.code[patch + 1] as u16;
        let jump = (high << 8) | low;
        assert_eq!(jump, 1); // 1 byte de cuerpo (Lorea)
    }

    #[test]
    fn chunk_disassemble_snapshot() {
        let mut chunk = Chunk::new("<test>", test_source());
        let span = Span::new(0, 1);
        chunk.emit_constant(Value::Numero(1.0), span.clone());
        chunk.emit_constant(Value::Numero(2.0), span.clone());
        chunk.emit_opcode(OpCode::Suma, span.clone());
        chunk.emit_opcode(OpCode::RetornarNada, span);

        insta::assert_snapshot!(chunk.to_string());
    }
}
