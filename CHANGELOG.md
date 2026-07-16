# Registro de cambios

Acá están todos los cambios importantes del proyecto, mi chanchito.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
y este proyecto sigue [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [No publicado]


## [0.3.0] - 2026-07-15

Esta es la release grande del ciclo: se acabo el interprete tree-walking, WN++ ahora compila a bytecode y corre en una VM propia. Junto con eso llega el sistema de modulos y la primera version de la biblioteca estándar.

### Agregado
- Migrar WN de interprete tree-walking a VM bytecode y agregar herramientas de inspeccion (wn ast, wn chunk) por @cuervolu ([#24](https://github.com/cuervolu/wn/pull/24))
- Agregar sistema de modulos (queri / ::) y biblioteca estandar de cadena y lista por @cuervolu ([#26](https://github.com/cuervolu/wn/pull/26))
- Agregar conversiones explicitas con numero() y texto() por @cuervolu ([#15](https://github.com/cuervolu/wn/pull/15))
- Agregar comando Uninstall al CLI, con formatos colorizados por @cuervolu
- Agregar tour interactivo de WN++ con WASM por @cuervolu ([#25](https://github.com/cuervolu/wn/pull/25))
- Habilitar el tour del lenguaje en produccion por @cuervolu ([#28](https://github.com/cuervolu/wn/pull/28))


### Cambiado
- Expandir WnError con variantes especificas de runtime para diagnosticos mas precisos por @cuervolu ([#14](https://github.com/cuervolu/wn/pull/14))
- Modularizar las funciones nativas de la VM y simplificar su registro por @cuervolu
- Cambiar la forma en que el tour maneja su contenido por @cuervolu
- Migrar la documentacion de Astro Starlight a Fumadocs por @cuervolu ([#27](https://github.com/cuervolu/wn/pull/27))


### Arreglado
- Corregir validacion de indices numericos en listas y textos por @jsgrrchg ([#20](https://github.com/cuervolu/wn/pull/20))
- Corregir el flag --force del updater por @jsgrrchg ([#19](https://github.com/cuervolu/wn/pull/19))
- Corregir layout del tour en moviles por @cuervolu
- Actualizar links rotos del README por @cuervolu


### Misceláneos
- Borrar flechitas sobrantes en comentarios de la stdlib y snapshots por @cuervolu
- Mantener documentacion y registro de cambios en español por @jsgrrchg ([#23](https://github.com/cuervolu/wn/pull/23))
- Pasar clippy y fmt sobre el codigo de la VM por @cuervolu
- Actualizar el gif de demo en assets por @cuervolu
- Agregar logo del proyecto por @cuervolu
- Agregar logo y creditos en el README por @cuervolu
- Juntar los deploys de tour y docs en un solo flujo de CI por @cuervolu
- Apuntar el despliegue a Cloudflare Workers por @cuervolu
- Arreglar orden del flujo de CI por @cuervolu
- Arreglar error de merge en CI por @cuervolu


## [0.2.1] - 2026-05-27


### Cambiado
- Renombrar builtin altiro a lorea por @cuervolu ([#13](https://github.com/cuervolu/wn/pull/13))


## [wn-v0.1.0] - 2026-05-25


### Arreglado
- Excluir piola-core de los artefactos distribuibles de cargo-dist por @cuervolu


### Misceláneos
- Arreglar formato de la tabla de archivos en el README por @cuervolu
- Agregar plantilla para PR por @cuervolu
- Agregar guía de contribución en CONTRIBUTING.md por @cuervolu
- Agregar configuración de CI para formateo, análisis estático y pruebas por @cuervolu
- Migrar marca y docs a WN++ por @jsgrrchg
- Actualizar instaladores y plantillas a WN++ por @jsgrrchg
- Actualizar flujo de publicación a WN++ por @jsgrrchg
- Migrar Piola a WN++ (#11) por @cuervolu ([#11](https://github.com/cuervolu/wn/pull/11))


## [piola-core-v0.1.0] - 2026-05-25


### Arreglado
- Impedir redeclaración de constantes duro en el mismo alcance (#9) por @cuervolu ([#9](https://github.com/cuervolu/wn/pull/9))


### Misceláneos
- Actualizar la configuración de publicación por @cuervolu
- Actualizar instrucciones de instalación en el README por @cuervolu
- Ajustes de instaladores y de packages por @cuervolu


## [piola-v0.1.0] - 2026-05-20


### Agregado
- Agregar documentación de control de flujo, manejo de errores, funciones, tipos de datos y variables por @cuervolu


### Misceláneos
- Agregar script auxiliar para publicaciones por @cuervolu
- Actualizar README con instrucciones de instalación para macOS, Linux y Windows por @cuervolu
- Actualizar versión de Node.js en el flujo de despliegue de documentación por @cuervolu
- Corregir CI por @cuervolu
- Agregar archivo de licencia MIT por @cuervolu
- Añadir un GIF de demo y actualizar el archivo README por @cuervolu
- Agregar hoja de ruta de Piola por @cuervolu
- Actualizar links del README por @cuervolu
- Actualizar el formato y el contenido del registro de cambios (#5) por @cuervolu ([#5](https://github.com/cuervolu/wn/pull/5))
- Actualizar la configuración de CI e incorporar la compatibilidad con Dependabot por @cuervolu


## [0.1.0]


### Agregado
- Agregar sentencias de control de flujo 'devolver', 'cortala' y 'sigue' con manejo de errores por @cuervolu
- Agregar comando de actualización automática (piola update) por @cuervolu


### Arreglado
- Renombrar binario de piola-cli a piola por @cuervolu
- Agregar Cross.toml para OpenSSL en compilación cruzada a aarch64 por @cuervolu


### Cambiado
- Agregar miette y mejorar el REPL por @cuervolu


### Misceláneos
- Agregar documentación inicial y filosofía del lenguaje Piola por @cuervolu
- Agregar documento de diseño de funcionalidades para el lenguaje Piola por @cuervolu
- Agregar flujo de CI para publicaciones por @cuervolu
- Agregar cliff.toml para generar el registro de cambios por @cuervolu


[unreleased]: https://github.com/cuervolu/wn/compare/v0.3.0..HEAD
[0.3.0]: https://github.com/cuervolu/wn/compare/v0.2.1..v0.3.0
[0.2.1]: https://github.com/cuervolu/wn/compare/wn-v0.1.0..v0.2.1
[wn-v0.1.0]: https://github.com/cuervolu/wn/compare/piola-core-v0.1.0..wn-v0.1.0
[piola-core-v0.1.0]: https://github.com/cuervolu/wn/compare/piola-v0.1.0..piola-core-v0.1.0
[piola-v0.1.0]: https://github.com/cuervolu/wn/compare/v0.1.0..piola-v0.1.0

<!-- generado por git-cliff -->
