---
title: Operadores y expresiones
section: El lenguaje
---

Aritmética, comparación y lógica. Casi todo funciona como te lo imaginái.

## Aritméticos

Los operadores básicos funcionan como en cualquier lenguaje. El módulo (`%`) devuelve el resto de una división.

```wn
wea entradas = 3500
wea descuento = 500
wea precio_final = entradas - descuento   //  3000

wea personas = 7
wea costo_por_persona = precio_final / personas   // 428.571...

wea resto = 10 % 3   // 1
```

La precedencia es la de siempre: `*` y `/` antes que `+` y `-`. Usa paréntesis para forzar otro orden.

```wn
lorea(2 + 3 * 4)     // 14
lorea((2 + 3) * 4)   // 20
```

## Comparación

Devuelven `verdad` o `falso`.

```wn
wea ano = 1995

lorea(ano == 1995)   // verdad
lorea(ano != 2000)   // verdad
lorea(ano > 2000)    // falso
lorea(ano <= 1995)   // verdad
```

## Lógicos

`y`, `o`, y `no` — en espanol, sin símbolos.

```wn
wea tiene_entrada = verdad
wea es_mayor = verdad

lorea(tiene_entrada y es_mayor)    // verdad  (ambas deben ser verdad)
lorea(tiene_entrada o falso)       // verdad  (basta una)
lorea(no tiene_entrada)            // falso
```

Se usan principalmente para construir condiciones compuestas, que verás en la próxima lección, no te apures tanto ctm.

## Concatenación de texto

El operador `+` con texto une strings. Si un lado es texto, el otro debe serlo también — usa `texto()` para convertir.

```wn
wea banda = "Los Bunkers"
wea ano = 1999

lorea("Fundados en " + texto(ano))         // "Fundados en 1999"
lorea(banda + " — " + texto(ano))          // "Los Bunkers — 1999"
```
