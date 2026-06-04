## mientras

Repite un bloque mientras la condición sea `verdad`. Úsalo cuando no sabes de antemano cuántas veces vas a iterar.

```wn
wea intentos = 0

mientras (intentos < 3) {
  lorea("Intento " + texto(intentos + 1))
  intentos = intentos + 1
}
```

Si la condición nunca se vuelve `falso`, el bucle no termina. Siempre asegúrate de que algo dentro del bloque cambie el estado.

## para

Itera sobre cada elemento de una lista. Úsalo cuando tienes una colección y quieres procesarla completa.

```wn
wea regiones = ["Atacama", "Valparaíso", "Metropolitana", "Biobío"]

para (region en regiones) {
  lorea("Región: " + region)
}
```

La variable `region` solo existe dentro del bloque — es nueva en cada vuelta.

## cortala

Sale del bucle inmediatamente, sin terminar la iteración actual ni las siguientes.

```wn
wea numeros = [3, 7, 2, 9, 1, 5]
wea encontrado = falso

para (n en numeros) {
  cachai (n == 9) {
    encontrado = verdad
    cortala
  }
}

lorea("¿Encontrado? " + texto(encontrado))
```

## sigue

Salta el resto del bloque en la vuelta actual y pasa a la siguiente.

```wn
wea notas = [45, 72, 38, 61, 55, 90]

para (nota en notas) {
  cachai (nota < 55) {
    sigue
  }
  lorea("Aprobado con " + texto(nota))
}
```

`cortala` y `sigue` funcionan igual dentro de `mientras`.
