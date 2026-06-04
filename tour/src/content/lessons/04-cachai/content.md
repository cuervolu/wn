`cachai` evalúa una condición y ejecuta un bloque si es `verdad`.

```wn
wea nota = 55

cachai (nota >= 60) {
  lorea("Aprobado")
}
```

Si la condición es `falso`, el bloque se salta y no pasa nada.

## si no

Para ejecutar algo cuando la condición no se cumple, agrega `si no`:

```wn
wea nota = 45

cachai (nota >= 60) {
  lorea("Aprobado")
} si no {
  lorea("Reprobado")
}
```

## Condiciones anidadas

Para evaluar más de dos casos, anida un `cachai` dentro del `si no`:

```wn
wea nota = 72

cachai (nota >= 90) {
  lorea("Distinción")
} si no {
  cachai (nota >= 60) {
    lorea("Aprobado")
  } si no {
    lorea("Reprobado")
  }
}
```

## Condiciones compuestas

Aquí entran los operadores lógicos de la lección anterior:

```wn
wea edad = 22
wea tiene_carnet = verdad

cachai (edad >= 18 y tiene_carnet) {
  lorea("Puede entrar")
} si no {
  lorea("No puede entrar")
}
```

## Variables declaradas adentro

Recuerda que las variables declaradas dentro de un bloque `cachai` no existen afuera:

```wn
wea temperatura = 28

cachai (temperatura > 25) {
  wea comentario = "Hace calor en Santiago"
  lorea(comentario)
}

// lorea(comentario)   Error: 'comentario' no existe acá
```
