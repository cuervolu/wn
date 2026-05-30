## Descripción

<!-- Qué cambia y por qué. Si el cambio no es obvio, explica el contexto. -->

Closes #<!-- número del issue -->

## Tipo de cambio

- [ ] Corrección de bug
- [ ] Nueva funcionalidad
- [ ] Refactor (sin cambio de comportamiento)
- [ ] Cambio en el lenguaje (sintaxis, semántica, tokens)
- [ ] Documentación
- [ ] CI / herramientas

## Qué cambia en el lenguaje (si aplica)

<!-- Llena esta sección solo si modificas lexer, parser, intérprete, o AST. -->

**Antes:**
```wn

```

**Después:**
```wn

```

## Pruebas

<!-- Describe cómo probaste el cambio. -->

- [ ] Agregué tests nuevos
- [ ] Los tests existentes pasan (`cargo test`)
- [ ] Probé manualmente con un archivo `.cl`

## Lista de verificación

- [ ] `cargo clippy` sin advertencias
- [ ] `cargo fmt` aplicado
- [ ] Si el PR toca el lexer, parser, o AST: corrí `cargo nextest` y revisé snapshots nuevos con `cargo insta review`
