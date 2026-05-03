# Filosofía de Piola

> *Piola: tranquilo, hábil, sin hacer ruido, de panini.*

## Identidad chilena

La identidad de Piola no es un chiste. No es poner palabras en español solo para que quede gracioso. Es un intento genuino de construir algo que **se sienta natural para alguien que creció hablando chileno**.

El español tiene estructura propia. Tiene ritmo propio. Y resulta que ese ritmo cabe perfectamente en un lenguaje de programación, si uno se toma el tiempo de diseñarlo bien.

```
wea x = 10
pega sumar(a, b) {
  a + b
}
altiro(sumar(x, 5))
```

`wea` no es broma. Es la palabra más versátil del español chileno, y eso la hace perfecta para representar una variable: un *algo* al que le damos nombre. `pega` es trabajo, lo que hace una función. `altiro` es inmediatamente, ya, ahora, lo que hace `print`.

Los errores también hablan chileno:

```
Error: No podi sumar un 'numero' con un 'texto' pedazo de saco wea.
Error: La wea 'x' no existe papito.
Error: Te fuiste al chancho, el índice 10 no existe en la lista (largo: 2).
```

El tono es intencionado. Un error que te hace reír igual te está diciendo exactamente qué salió mal.

## Decisiones de sintaxis

Cada keyword de Piola fue elegida con criterio. No son traducciones literales del inglés ni palabras random en chileno. Son palabras que ya tienen un significado concreto en la cultura local, y ese significado se mapea bien al concepto técnico que representan.

**`wea`** para variables. Una variable es un contenedor para cualquier cosa, un "algo" sin forma fija. En chileno, *wea* cumple exactamente ese rol: puede ser cualquier objeto, situación o concepto. Es la palabra más polivalente del idioma, y por eso es la palabra correcta.

**`pega`** para funciones. Una función es un bloque de trabajo que se le encarga a la máquina. *Pega* en Chile es trabajo, labor, lo que haces cuando te contratan. Defines una pega, y después la mandas a hacer.

**`duro`** para constantes. Algo duro no se dobla, no cambia. La constante de pi no negocia su valor. El nombre es directo y no necesita explicación.

**`cachai`** para condicionales. Es una pregunta implícita: ¿captas que esta condición es verdad? Si sí, ejecuta el bloque. La interrogación está en el nombre. Ningún otro keyword captura mejor la idea de "evalúa si esto es cierto antes de continuar".

**`altiro`** para imprimir. Significa "al tiro", de inmediato. Imprimir es la acción más inmediata que tiene un programa para comunicarse con el exterior.

**`ojo`** y **`cago`** para manejo de errores. *Ojo* es advertencia, atención, cuidado. Metes en `ojo` el código que puede fallar. Si falla, `cago` es exactamente lo que pasó: algo se fue al tacho, y ahora lo manejamos.

**`si no`** para el else. Dos palabras en vez de una, porque así se dice en español. "Si la condición es verdad, haz esto. Si no, haz lo otro." Es la transcripción directa del razonamiento.

Lo que se evitó fue traducir mecánicamente: `si` para `if`, `entonces` para `then`, `mientras_que` para `while`. Eso resulta en código que suena como un manual de instrucciones pasado por Google Translate. Piola prefiere palabras que un chileno diría de verdad.

## Influencias y rechazos

Piola no nació del vacío. Tomó cosas de otros lenguajes, pero también rechazó cosas de forma explícita.

**De Lua** viene la idea de que un lenguaje liviano puede ser completo sin ser complicado. Lua demuestra que no necesitas un runtime enorme para tener un lenguaje funcional.

**De Python** viene la preferencia por el retorno implícito en funciones y la idea de que el código debe leerse casi como prosa. También la filosofía de que los mensajes de error deben orientar en vez de intimidar.

**De Ruby** viene el tono. Ruby tiene personalidad, y su documentación y errores lo muestran. Piola toma eso en serio: un lenguaje puede tener carácter propio sin sacrificar claridad técnica.

**De Rust** (el lenguaje en que está implementado) viene la obsesión con que los errores sean informativos y apunten exactamente al problema. El compilador de Rust es famoso por sus mensajes de error. Piola aspira a tener esa misma precisión, con otro tono.

Lo que Piola rechaza conscientemente:

**El silencio de JavaScript**. JS convierte tipos de formas inesperadas, falla silenciosamente, y tiene reglas de coerción que nadie recuerda completas. Piola prefiere fallar fuerte y claro antes que ejecutar algo incorrecto sin avisar.

**La verbosidad de Java**. Declarar tipos en todos lados, `public static void main`, getters y setters para todo. Piola es dinámico y conciso porque el objetivo es que el código diga lo que hace, no que demuestre que siguió el procedimiento correcto.

**La magia implícita de ciertos frameworks**. Código que hace cosas "por ti" sin que entiendas qué está pasando. Piola ejecuta exactamente lo que el código dice que ejecute, nada más.

## Sobre el humor en los errores

El humor en los mensajes de error de Piola es una decisión técnica disfrazada de personalidad.

Un mensaje de error tiene que ser memorable. Si es genérico y frío, lo lees y lo cierras. Si tiene carácter, lo lees, te ríes, y lo recuerdas. Recordar el error es parte de no repetirlo.

Dicho eso, hay una línea que Piola respeta: el humor nunca reemplaza la información. El error siempre dice qué salió mal y dónde, primero. El tono viene después, como condimento, no como sustituto.

```
Error: No podi sumar un 'numero' con un 'texto' pedazo de saco wea.
```

Esa línea tiene tres partes: la operación que falló (`sumar`), los tipos involucrados (`numero`, `texto`), y el tono (`pedazo de saco wea`). Quita la última parte y el error sigue siendo útil. Quita las primeras dos y el error es solo un insulto sin contexto.

El tono también es contextual. No todos los errores necesitan humor. Piola calibra según la gravedad: los errores de tipo y de variable inexistente tienen personalidad, los errores de sistema son directos.

## Principios de diseño

Estos principios guían cada decisión del lenguaje. Cuando dos ideas entran en conflicto, estos principios son el desempate.

### 1. Lo que lees es lo que hay

El código Piola debe ser predecible. No hay magia implícita, no hay comportamiento oculto, no hay abstracciones que disfracen lo que realmente está ocurriendo.

Si una variable cambia dentro de un bloque, eso debe ser obvio por cómo está escrito, no un efecto secundario sorpresivo. Si una función puede fallar, eso debe ser visible en el código.

```
wea contador = 0
mientras (contador < 5) {
  contador = contador + 1   // modifica la wea del bloque padre
}
```

### 2. Los errores son parte del lenguaje

Un error en Piola no es solo un mensaje de falla, es una oportunidad de aprender. Cada mensaje de error debe decir qué salió mal, dónde (con el _span_ exacto apuntando al código), y por qué cuando el contexto lo justifica.

Y si puede decirlo con humor chileno sin perder precisión, mejor.

### 3. Simple primero, poderoso después

Piola prioriza tener una implementación simple y correcta por sobre tener muchas features. Cada cosa que se agrega debe justificar su complejidad de implementación.

Esto no significa que Piola sea pobre en features. Significa que cada feature existe porque resuelve un problema real que aparece al escribir código Piola, no porque "los otros lenguajes la tienen".

### 4. Dinámico por elección, no por descuido

Piola es de tipado dinámico. No porque sea más fácil de implementar (_aunque lo es_), sino porque permite entender primero cómo funciona la evaluación de expresiones, el scoping y el manejo de errores, sin la complejidad adicional de un sistema de tipos estático.

```
wea x = 42
x = "ahora soy texto"   // válido, la wea puede cambiar de tipo
```

Los errores de tipo ocurren en tiempo de ejecución. El lenguaje confía en que quien escribe código sabe lo que está haciendo, y cuando no es así, el mensaje de error lo dice sin rodeos (y a veces te funa por ello).

### 5. El pipeline es el aprendizaje

La arquitectura de Piola refleja las fases clásicas de un compilador:

```
Fuente → Lexer → Parser → AST → Intérprete → (Bytecode → VM)
```

Cada fase es un módulo separado con una interfaz clara. El objetivo es que leer el código fuente de Piola sea, en sí mismo, un recorrido por cómo funcionan los lenguajes de programación por dentro.