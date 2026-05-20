# Piola

Piola es un lenguaje de programación de propósito general, de tipado dinámico, implementado en Rust. Nació con dos objetivos que no se contradicen: ser una herramienta de aprendizaje sobre implementación de lenguajes, y tener una identidad chilena genuina.

No es un lenguaje de producción (todavía). Es un lenguaje para entender qué pasa por debajo, cómo un lexer convierte texto en tokens, cómo un parser construye un árbol, cómo un intérprete le da vida a ese árbol. Y todo eso, con la cadencia y el humor del famoso español chileno.

```
pega fibonacci(n) {
  cachai (n <= 1) {
    n
  } si no {
    fibonacci(n - 1) + fibonacci(n - 2)
  }
}

altiro(fibonacci(10))  // → 55
```

## Instalación

### macOS y Linux

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/cuervolu/piola/releases/latest/download/piola-cli-installer.sh | sh
```

### Windows (PowerShell)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/cuervolu/piola/releases/latest/download/piola-cli-installer.ps1 | iex"
```

### Verificar la instalación

```sh
piola --version
```

### Actualizar

```sh
piola update
```

### Instalación manual

Si prefieres no usar los scripts, descarga el binario para tu plataforma directamente desde [GitHub Releases](https://github.com/cuervolu/piola/releases/latest) y agrégalo a tu `PATH`.

| Plataforma            | Archivo                                              |
|-----------------------|------------------------------------------------------|
| macOS (Apple Silicon) | `piola-cli-aarch64-apple-darwin.tar.xz`              |
| macOS (Intel)         | `piola-cli-x86_64-apple-darwin.tar.xz`               |
| Linux x86_64          | `piola-cli-x86_64-unknown-linux-gnu.tar.xz`          |
| Linux ARM64           | `piola-cli-aarch64-unknown-linux-gnu.tar.xz`         |
| Windows 64-bit        | `piola-cli-x86_64-pc-windows-msvc.zip`               |

Cada archivo tiene un `.sha256` correspondiente para verificar la integridad.

---

## Uso

```sh
piola              # abre el REPL interactivo
piola programa.cl  # ejecuta un archivo
```

## El nombre

*Piola* en Chile tiene varios significados según el contexto. Puede ser alguien tranquilo y hábil, algo que funciona bien sin hacer bulla, o simplemente un elogio: "quedó piola".

El lenguaje aspira a ser las tres cosas: tranquilo de leer, hábil en lo que hace, y que cuando algo funciona, simplemente funcione, sin aspavientos.

Además, un restaurante que me gusta se llama así, ojalá no me demanden.

## Para quién es

Piola está pensado para quien quiere entender cómo funcionan los lenguajes de programación por dentro, sin necesitar un doctorado para arrancar.

Si alguna vez te preguntaste cómo hace Python para saber que `3 + 4 * 2` es `11` y no `14`, qué es exactamente un *scope*, cómo funciona un garbage collector, o qué pasa entre que escribes código y la máquina lo ejecuta — Piola está construido para que puedas responder esas preguntas leyendo su código fuente.

El intérprete está escrito en Rust, es deliberadamente simple, y cada fase del pipeline existe como un módulo separado y legible. No asumimos conocimiento de teoría formal de lenguajes. Asumimos que sabes programar y tienes curiosidad.

## Lo que Piola no es

**Piola no es un lenguaje de producción** (por ahora). No está optimizado para alto rendimiento, no tiene un ecosistema de librerías, no tiene garantías de estabilidad de API. Si necesitas eso, usa Python, Go, o Rust directamente.

**Piola no intenta representar el español en programación de forma general**. Hay otros lenguajes en español — algunos serios, algunos experimentales. Piola no compite con ellos. Piola es específicamente chileno, no genéricamente hispano.

**Piola no esconde su complejidad**, la expone. Si algo es difícil de implementar, el código lo muestra. No hay atajos que oculten cómo funciona el mecanismo real.

## Estado actual

El pipeline completo — lexer, parser, AST e intérprete tree-walking — está implementado y funcional. Las siguientes fases — compilador a bytecode, VM y garbage collector — están en el roadmap. El detalle de cada fase está en [`docs/roadmap`](https://cuervolu.github.io/piola/roadmap/).

## Construir desde el código fuente

Si quieres compilar Piola tú mismo necesitas [Rust](https://rustup.rs) 1.80 o superior.

```sh
git clone https://github.com/cuervolu/piola
cd piola
cargo build --release
./target/release/piola
```

## Contribuir

Piola es open source. Si te interesa contribuir, sea código, documentación, ejemplos, o simplemente feedback sobre qué se siente raro al escribir el lenguaje, eres bienvenido.

Lo más valioso que puedes hacer en esta etapa es escribir programas en Piola y reportar qué se siente natural y qué no. El lenguaje mejora con uso real, no solo con teoría.

Para entender la filosofía del proyecto antes de contribuir, lee [`docs/filosofia`](https://cuervolu.github.io/piola/filosofia/).