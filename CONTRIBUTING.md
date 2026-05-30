# Contribuyendo a WN++

Wena, gracias por querer contribuir. WN++ es un proyecto en etapa temprana, así que cada aporte cuenta, desde un typo en la doc hasta una funcionalidad nueva del lenguaje.

Antes de empezar, lee esto. No es largo.

## ¿Por dónde empiezo?

Depende de qué quieres cambiar:

**Abre un issue primero si tu cambio afecta el lenguaje:**
- Lexer, parser, o AST
- Sintaxis o semántica de WN++
- El intérprete o comportamiento en ejecución
- Mensajes de error

Los cambios al lenguaje tienen consecuencias en cadena. Vale la pena discutir el diseño antes de implementar.

**Puedes abrir un PR directamente para:**
- Typos o mejoras en la documentación
- Fixes de CI
- Refactors que no cambian comportamiento
- Mejoras a las herramientas internas

## Entorno de desarrollo

Necesitas tener instalado:

```bash
# Rust stable
rustup toolchain install stable

# Herramientas de Rust
cargo install cargo-nextest
cargo install cargo-insta
cargo install git-cliff
cargo install cargo-dist # (o cargo binstall cargo-dist)

# Clippy viene con rustup, pero por si acaso
rustup component add clippy

# Para trabajar en la documentación
# https://pnpm.io/installation
pnpm install  # desde la carpeta docs/
```

Para compilación cruzada entre plataformas también necesitas [`cross`](https://github.com/cross-rs/cross), pero eso es solo si trabajas en el flujo de publicación.

## Comandos útiles

```bash
# Correr los tests
cargo nextest run

# Revisar snapshots nuevos o modificados (después de cargo nextest run)
cargo insta review

# Linting
cargo clippy --all-targets --all-features

# Formatear
cargo fmt

# Documentación (desde docs/)
pnpm dev
```

## Commits convencionales

Este proyecto usa conventional commits. El tipo va en inglés, la descripción en español.

```
feat(lexer): agregar soporte para strings interpolados
fix(parser): corregir precedencia del operador 'no'
chore(ci): actualizar versión de actions/checkout
doc(readme): agregar instrucciones de instalación en Windows
```

Los tipos que reconoce el CHANGELOG:

| Tipo                                  | Aparece en el CHANGELOG como |
|---------------------------------------|------------------------------|
| `feat`                                | Agregado                     |
| `fix`                                 | Arreglado                    |
| `refactor`, `perf`, `style`           | Cambiado                     |
| `remove`, `revert`                    | Eliminado                    |
| `security`                            | Seguridad                    |
| `deprecat`                            | Obsoleto                     |
| `doc`, `test`, `chore`, `ci`, `build` | Misceláneos                  |
| `wip`                                 | No aparece (se omite)        |

El alcance es opcional pero recomendado cuando el cambio está acotado a una parte específica: `lexer`, `parser`, `ast`, `interpreter`, `cli`, `repl`, `docs`.

## Pull requests

1. Haz fork del repo y trabaja en una rama con nombre descriptivo: `feat/strings-interpolados`, `fix/precedencia-not`.
2. Un PR por cambio. No mezcles features con refactors. (a veces hasta yo soy porfiado con esa wea, pero es mejor para la revisión y el historial).
3. Asegúrate de que pasa el CI antes de pedir revisión:
   ```bash
   cargo nextest run
   cargo clippy --all-targets --all-features
   cargo fmt --check
   ```
4. Si modificaste el lexer, parser, o AST, corre `cargo nextest run` y revisa los snapshots con `cargo insta review`. Los archivos `.snap` actualizados van en el mismo commit.
5. Usa el PR template. El CHANGELOG se genera automáticamente, no edites `CHANGELOG.md` manualmente.

## Estilo de código

- Rust edition `2024`.
- `cargo fmt` es obligatorio. Si el CI falla por formato, no se revisa el PR.
- Prefiere `match` exhaustivo sobre cadenas de `if let` al traversar el AST.
- Todo token y nodo del AST debe tener `Span`. Sin excepciones.
- Los mensajes de error en ejecución van en castellano chileno. Si agregas uno nuevo, que se sienta natural al lado de los existentes.

## ¿Dudas?

Abre una [discusión](https://github.com/cuervolu/wn/discussions) en vez de un issue. Los issues son para bugs y features concretas.
