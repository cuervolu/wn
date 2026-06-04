Cuando algo explota en tiempo de ejecución, WN++ lo dice a la chilena. `ojo`/`cago` te permite capturar esos errores antes de que corten el programa.

## Sintaxis básica

```wn
ojo {
  wea resultado = 10 / 0
} cago(error) {
  lorea("Atrapado: " + error)
}
// Atrapado: Weon, no se puede dividir por cero.
```

El bloque `ojo` envuelve el código que puede fallar. Si algo revienta, `cago` lo atrapa y el programa sigue corriendo.

## El error es un texto

La variable que declaras en `cago(e)` es un `texto` con el mensaje del error. Puedes llamarla como quieras y usarla como cualquier string:

```wn
ojo {
  lorea(variable_inexistente)
} cago(e) {
  lorea("Oe: " + e)
}
// Oe: La wea 'variable_inexistente' no existe papito.
```

## Errores comunes que puedes capturar

Cualquier error en tiempo de ejecución es capturable:

```wn
// Operación entre tipos incompatibles
ojo {
  wea x = "fósforos" * 2
} cago(e) { lorea(e) }

// Índice fuera de rango
wea lista = ["uno", "dos", "tres"]
ojo {
  lorea(lista[99])
} cago(e) { lorea(e) }

// Intentar modificar una constante
duro PRECIO = 800
ojo {
  PRECIO = 750
} cago(e) { lorea(e) }
```

## Lo que `ojo` NO captura

`cortala`, `sigue` y `devolver` no son errores — son señales de control de flujo. Pasan a través del `ojo` sin ser interceptadas:

```wn
para (i en [1, 2, 3, 4, 5]) {
  ojo {
    cachai (i == 3) {
      cortala  // sale del bucle, el cago no lo ve
    }
    lorea(i)
  } cago(e) {
    lorea("error: " + e)
  }
}
// 1
// 2
// (se corta en 3, el cago nunca ejecuta)
```

Esto es intencional: si `devolver` fuera capturable, no podrías retornar desde dentro de un bloque de manejo de errores. El `ojo` solo atrapa errores reales, no decisiones de flujo.

## Pruébalo

El editor tiene tres ejemplos. Ejecútalos y después intenta romperlos: cambia los valores, usa índices imposibles, intenta capturar un `cortala`.