# WN++

<p align="center">
  <img src="assets/logo.png" alt="Logo de WN++" width="220" />
</p>

<p align="center">
  <img src="assets/demo.gif" alt="Demo del REPL de WN++" width="700" />
</p>

WN++ es un lenguaje de programación de propósito general, de tipado dinámico, implementado en Rust. Nació con dos objetivos que no se contradicen: ser una herramienta de aprendizaje sobre implementación de lenguajes, y tener una identidad chilena genuina.

No es un lenguaje de producción (todavía). Es un lenguaje para entender qué pasa por debajo, cómo un lexer convierte texto en tokens, cómo un parser construye un árbol, cómo un intérprete le da vida a ese árbol. Y todo eso, con la cadencia y el humor del famoso español chileno.

```wn
pega fibonacci(n) {
  cachai (n <= 1) {
    n
  } si no {
    fibonacci(n - 1) + fibonacci(n - 2)
  }
}

lorea(fibonacci(10))  //  55
```

## Instalación

### macOS y Linux

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/cuervolu/wn/releases/latest/download/wn-cli-installer.sh | sh
```

### Windows (PowerShell)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/cuervolu/wn/releases/latest/download/wn-cli-installer.ps1 | iex"
```

### Verificar la instalación

```sh
wn --version
```

### Actualizar

```sh
wn update
```

### Instalación manual

Si prefieres no usar los scripts, descarga el binario para tu plataforma directamente desde [GitHub Releases](https://github.com/cuervolu/wn/releases/latest) y agrégalo a tu `PATH`.

| Plataforma            | Archivo                                   |
| --------------------- | ----------------------------------------- |
| macOS (Apple Silicon) | `wn-cli-aarch64-apple-darwin.tar.gz`      |
| macOS (Intel)         | `wn-cli-x86_64-apple-darwin.tar.gz`       |
| Linux x86_64          | `wn-cli-x86_64-unknown-linux-gnu.tar.gz`  |
| Linux ARM64           | `wn-cli-aarch64-unknown-linux-gnu.tar.gz` |
| Windows 64-bit        | `wn-cli-x86_64-pc-windows-msvc.zip`       |

Cada archivo tiene un `.sha256` correspondiente para verificar la integridad.

---

## Uso

```sh
wn              # abre el REPL interactivo
wn programa.cl  # ejecuta un archivo
```

## El nombre

_WN++_ se pronuncia como "wena plus plus": corto, chileno y fácil de escribir como comando `wn`.

El nombre mantiene la intención original del proyecto: un lenguaje tranquilo de leer, directo para aprender cómo funciona un intérprete por dentro, y con una identidad chilena que no se siente pegada encima.

El `++` es un guiño a la idea de ir construyéndolo por capas: primero intérprete, después herramientas, bytecode, VM y todo lo que venga cuando el lenguaje se gane ese crecimiento.

## Para quién es

WN++ está pensado para quien quiere entender cómo funcionan los lenguajes de programación por dentro, sin necesitar un doctorado para arrancar.

Si alguna vez te preguntaste cómo hace Python para saber que `3 + 4 * 2` es `11` y no `14`, qué es exactamente un _scope_, cómo funciona un garbage collector, o qué pasa entre que escribes código y la máquina lo ejecuta — WN++ está construido para que puedas responder esas preguntas leyendo su código fuente.

El intérprete está escrito en Rust, es deliberadamente simple, y cada fase del flujo existe como un módulo separado y legible. No asumimos conocimiento de teoría formal de lenguajes. Asumimos que sabes programar y tienes curiosidad.

## Lo que WN++ no es

**WN++ no es un lenguaje de producción** (por ahora). No está optimizado para alto rendimiento, no tiene un ecosistema de librerías, no tiene garantías de estabilidad de API. Si necesitas eso, usa Python, Go, o Rust directamente.

**WN++ no intenta representar el español en programación de forma general**. Hay otros lenguajes en español — algunos serios, algunos experimentales. WN++ no compite con ellos. WN++ es específicamente chileno, no genéricamente hispano.

**WN++ no esconde su complejidad**, la expone. Si algo es difícil de implementar, el código lo muestra. No hay atajos que oculten cómo funciona el mecanismo real.

## Estado actual

El flujo completo — lexer, parser, AST e intérprete tree-walking — está implementado y funcional. Las siguientes fases — compilador a bytecode, VM y recolector de basura — están en la hoja de ruta. El detalle de cada fase está en [`docs/roadmap`](https://wnpp.cl/docs/roadmap).

## Construir desde el código fuente

Si quieres compilar WN++ tú mismo necesitas [Rust](https://rustup.rs) 1.80 o superior.

```sh
git clone https://github.com/cuervolu/wn
cd wn
cargo build --release
./target/release/wn
```

## Contribuir

WN++ es de código abierto. Si te interesa contribuir, sea código, documentación, ejemplos, o simplemente comentarios sobre qué se siente raro al escribir el lenguaje, eres bienvenido.

Lo más valioso que puedes hacer en esta etapa es escribir programas en WN++ y reportar qué se siente natural y qué no. El lenguaje mejora con uso real, no solo con teoría.

Para entender la filosofía del proyecto antes de contribuir, lee [`docs/filosofia`](https://wnpp.cl/docs/filosofia/).

## Créditos

- [iriata18](http://instagram.com/iriata18), por el diseño del logo y la identidad visual de WN++.
- [r/ChileIT](https://www.reddit.com/r/chileIT/), por el apoyo y feedback que dieron cuando se anunció el proyecto. De esa comunidad salió el nombre, el keyword `lorea`, y muchas ideas sobre la identidad chilena del lenguaje.
