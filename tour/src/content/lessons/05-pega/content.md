---
title: Funciones
section: El lenguaje
---

Una función se define con `pega`. Agrupa código que puedes reutilizar.

```wn
pega saludar(nombre) {
  lorea("Wena, " + nombre + "!")
}

saludar("Macarena")   //  Wena, Macarena!
saludar("Rodrigo")    //  Wena, Rodrigo!
```

## Retorno implícito

No hay `return`. La última expresión del cuerpo es el valor que devuelve la función.

```wn
pega propina(cuenta) {
  cuenta * 0.1
}

wea total = 8500 + propina(8500)
lorea("Total con propina: " + texto(total))   //  9350.0
```

Si la última línea es una llamada a `lorea` u otra instrucción que no produce valor, la función devuelve `nada`. En general, si quieres que una función devuelva algo, asegúrate de que su última línea sea una expresión.

## Sin parámetros

Una función puede no recibir nada:

```wn
pega despedida() {
  "¡Chao pescao!"
}

lorea(despedida())   //  ¡Chao pescao!
```

## Múltiples parámetros

```wn
pega precio_con_descuento(precio, porcentaje) {
  precio - (precio * porcentaje / 100)
}

lorea(precio_con_descuento(12000, 20))   //  9600.0
```

## Recursión

Una función puede llamarse a sí misma. El caso base evita que el loop sea infinito.

```wn
pega factorial(n) {
  cachai (n <= 1) {
    1
  } si no {
    n * factorial(n - 1)
  }
}

lorea(factorial(5))   //  120
```

## Scope

Las variables declaradas dentro de una `pega` no existen afuera. Las variables del exterior sí son visibles desde adentro.

```wn
duro IVA = 0.19

pega precio_final(neto) {
  neto + (neto * IVA)   // IVA es visible aquí
}

lorea(precio_final(10000))   //  11900.0
```
