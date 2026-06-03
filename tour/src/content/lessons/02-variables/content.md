# Variables y constantes

En WN++ hay dos formas de guardar valores.

## `wea` — variables mutables

```wn
wea x = 10
wea nombre = "Juanito"
x = x + 1   // puedes reasignar
```

`wea` es como `var` o `let` en otros lenguajes. El valor puede cambiar.

## `duro` — constantes

```wn
duro PI = 3.14159
duro VERSION = "0.2.1"
```

`duro` es como `const`. Si intentas reasignar, el intérprete te reclama.

## Tipos disponibles

WN++ es dinámico, pero los tipos existen:

- `numero` — cualquier número (`10`, `3.14`, `-5`)
- `texto` — strings (`"wena"`, `'también funciona'`)
- `booleano` — `verdad` o `falso`
- `nada` — ausencia de valor (como `null` o `None`)

## Pruébalo

El editor ya tiene ejemplos. Modifica los valores y ejecuta.