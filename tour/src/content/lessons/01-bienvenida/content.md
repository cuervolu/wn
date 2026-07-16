---
title: Bienvenida
section: Introducción
---

Wn++ es un lenguaje de programación dinámico con sabor chileno, escrito en Rust. 
Este tour es la forma más rápida de cacharlo, sin instalar nada.

## Cómo funciona el playground

El editor de la derecha tiene código `wn` editable de verdad. Cuando presiones **▶ ejecutar**, el código corre altiro acá en tu navegador, no hay servidor metío en la wea.

El resultado aparece en el panel output de abajo. Si hay un error, también cae ahí, con la línea y el mensaje correspondiente.

> **tip**
>
> Todo es editable: cambiá los valores, rompé cosas a propósito y vuelve a ejecutar. Así se cacha más rápido. El botón reset te devuelve el código original de la lección.

### stdin

Algunas lecciones usan `pregunta()` para leer input. Si el código tiene un `pregunta()`,
activa el panel de **stdin** con el botón `stdin` del editor y escribe el valor antes de ejecutar.

## Una probadita del lenguaje

WN++ usa palabras en español (y chilenismos) como palabras clave:

```wn
// Variables y constantes
wea nombre = "Juanito"     // mutable
duro PI = 3.14159          // inmutable

// Función con retorno implícito
pega saludar(x) {
  "Wena, " + x + "!"
}

// Condicional
cachai (PI > 3) {
  lorea(saludar(nombre))   // Wena, Juanito!
} si no {
  lorea("Algo está muy mal")
}
```

`lorea` imprime en pantalla. `cachai` es el `if`. `pega` define una función.
La última expresión de una `pega` es el valor de retorno, sin `return` explícito.

## Por dónde seguir

Las lecciones de la izquierda están ordenadas. Puedes ir en orden o saltar a lo que te interese.
Empieza con **Variables y constantes** cuando estés listo.