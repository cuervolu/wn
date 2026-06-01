# Documentación de WN++

Este directorio contiene el sitio de documentación oficial de WN++, construido con Astro y Starlight.

## Estructura

```text
.
├── public/
├── src/
│   ├── assets/
│   ├── content/
│   │   └── docs/
│   └── content.config.ts
├── astro.config.mjs
├── package.json
└── tsconfig.json
```

Las páginas de documentación viven en `src/content/docs/`. Cada archivo `.md` o `.mdx` se publica como una ruta del sitio según su nombre.

Los recursos versionados de la documentación van en `src/assets/`. Los archivos estáticos que deben servirse tal cual, como favicons, van en `public/`.

## Comandos

Ejecuta estos comandos desde este directorio:

| Comando                  | Acción                                                |
|--------------------------|-------------------------------------------------------|
| `pnpm install`           | Instala las dependencias de la documentación.         |
| `pnpm dev`               | Levanta el sitio local en `localhost:4321`.           |
| `pnpm build`             | Genera el sitio de producción en `./dist/`.           |
| `pnpm preview`           | Previsualiza el build generado antes de publicarlo.   |
| `pnpm astro ...`         | Ejecuta comandos de Astro, como `astro check`.        |
| `pnpm astro -- --help`   | Muestra la ayuda de la CLI de Astro.                  |

## Mantención

La documentación oficial de WN++ se escribe principalmente en español. Mantén el tono claro, cercano y consistente con el resto del proyecto.

Si agregas una página nueva, actualiza la navegación en `astro.config.mjs` cuando corresponda.
