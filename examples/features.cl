// Variables y constantes
wea x = 10
duro PI = 3.1415

// Operadores aritméticos
wea suma = x + 5
wea resta = x - 3
wea producto = x * 2
wea division = x / 4
wea modulo = x % 3

lorea("Suma: " + suma)
lorea("Resta: " + resta)
lorea("Producto: " + producto)
lorea("División: " + division)
lorea("Módulo: " + modulo)

// Booleanos y condicionales
wea edad = 20
cachai (edad >= 18 y edad < 65) {
  lorea("Estai en edad pa trabajar")
} si no {
  lorea("No estai en edad pa trabajar")
}

// Bucle mientras
wea contador = 0
mientras (contador < 3) {
  lorea("Contador: " + contador)
  contador = contador + 1
}

// Listas
wea mi_lista = ["uno", "dos", "tres"]
lorea("Largo lista: " + largo(mi_lista))
para (item en mi_lista) {
  lorea(item)
}

// Mapas
wea persona = {"nombre": "Zalo", "edad": 69}
lorea("Nombre: " + persona["nombre"])
lorea("Edad: " + persona["edad"])

// Tipos
lorea(cachar(42))
lorea(cachar("wena"))
lorea(cachar(verdad))
lorea(cachar(nada))
lorea(cachar([1, 2]))

// Manejo de errores
ojo {
  wea resultado = 10 / 0
} cago(error) {
  lorea("Capturé error: " + error)
}