En WN++, toda variable se declara con `wea`. Es mutable: puedes cambiarle el valor cuando quieras.

```wn
wea banda = "31 Minutos"
wea integrantes = 7
banda = "Los Prisioneros"   // válido, wea es mutable
```

Las constantes usan `duro`. Una vez asignadas, no cambian, intentarlo es un error en runtime.

```wn
duro ANO_DEBUT = 1999
ANO_DEBUT = 2000   // Error
```

## Tipos de datos

WN++ es dinámico: la variable no tiene tipo fijo, el _valor_ sí.

```wn
wea x = 42             // numero  (siempre f64 internamente)
wea y = "Tulio Triviño" // texto
wea z = verdad         // booleano  (verdad / falso)
wea w = nada           // nada  (ausencia de valor)
```

Puedes consultar el tipo de cualquier valor con `cachar()`:

```wn
lorea(cachar(42))        //  "numero"
lorea(cachar("Tulio"))   //  "texto"
lorea(cachar(verdad))    //  "booleano"
lorea(cachar(nada))      //  "nada"
```

## Scope de bloque

Las variables `wea` viven en el bloque `{}` donde se declaran. Afuera no existen.

```wn
wea popularidad = 100

cachai (popularidad > 50) {
  wea estado = "legendario"
  lorea(estado)    //  "legendario"
}

lorea(estado)      // Error: 'estado' no existe acá
```

`duro` funciona igual — scope de bloque, pero inmutable.

## Conversión de tipos

Para convertir entre tipos, usa `numero()` y `texto()`:

```wn
wea ano = "2001"
wea siguiente = numero(ano) + 1   //  2002

wea canciones = 12
lorea("Tiene " + texto(canciones) + " canciones")
```

Mezclar tipos sin convertir explícitamente es un error, WN++ no hace coerciones silenciosas.
