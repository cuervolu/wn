## Listas

Una lista es una colección ordenada de valores. Se define con `[]` y puede mezclar tipos.

```wn
wea frutas = ["palta", "mango", "lúcuma"]
wea mixta  = [1, "dos", verdad, nada]
```

Accede a un elemento por su índice — parte desde `0`:

```wn
lorea(frutas[0])   // → palta
lorea(frutas[2])   // → lúcuma
```

Los índices negativos cuentan desde el final:

```wn
lorea(frutas[-1])  // → lúcuma  (el último)
lorea(frutas[-2])  // → mango
```

Puedes modificar un elemento existente asignando al índice:

```wn
frutas[1] = "chirimoya"
lorea(frutas[1])   // → chirimoya
```

Acceder a un índice que no existe es un error en runtime.

Para recorrer una lista usa `para`, que ya viste en la lección de bucles:

```wn
para (fruta en frutas) {
  lorea(fruta)
}
```

`largo()` devuelve la cantidad de elementos:

```wn
lorea(largo(frutas))   // → 3
```

## Mapas

Un mapa almacena pares clave-valor. Las claves son siempre texto.

```wn
wea persona = {
  "nombre": "Valentina",
  "edad": 24,
  "ciudad": "Valparaíso"
}
```

Accede a un valor por su clave:

```wn
lorea(persona["nombre"])   // → Valentina
lorea(persona["edad"])     // → 24
```

Acceder a una clave que no existe es un error en runtime.

Puedes modificar un valor existente o agregar una clave nueva:

```wn
persona["edad"] = 25              // modifica
persona["profesion"] = "diseñadora"   // agrega clave nueva
```

Los mapas no son iterables directamente con `para` — si necesitas recorrer un mapa, guarda las claves en una lista aparte.

## Estructuras anidadas

Las listas y los mapas pueden contener cualquier tipo, incluyendo otros mapas y listas:

```wn
wea equipo = {
  "nombre": "Colo-Colo",
  "titulos": 33,
  "jugadores": ["Leao", "Palacios", "Bolados"]
}

lorea(equipo["nombre"])           // → Colo-Colo
lorea(equipo["jugadores"][0])     // → Leao
```
