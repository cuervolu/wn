## Descripción

<!-- Qué cambia y por qué. Si el cambio no es obvio, explica el contexto. -->

Closes #<!-- número del issue -->

## Tipo de cambio

- [ ] Bug fix
- [ ] Nueva feature
- [ ] Refactor (sin cambio de comportamiento)
- [ ] Cambio en el lenguaje (sintaxis, semántica, tokens)
- [ ] Documentación
- [ ] CI / tooling

## Qué cambia en el lenguaje (si aplica)

<!-- Llena esta sección solo si modificas lexer, parser, intérprete, o AST. -->

**Antes:**
```piola

```

**Después:**
```piola

```

## Testing

<!-- Describe cómo probaste el cambio. -->

- [ ] Agregué tests nuevos
- [ ] Los tests existentes pasan (`cargo test`)
- [ ] Probé manualmente con un archivo `.cl`

## Checklist

- [ ] `cargo clippy` sin warnings
- [ ] `cargo fmt` aplicado
- [ ] Si el PR toca el lexer, parser, o AST: corrí `cargo nextest` y revise snapshots nuevos con `cargo insta review`