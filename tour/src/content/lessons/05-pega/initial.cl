// Retorno implícito: la última expresión se devuelve
pega saludar(nombre) {
  "Wena, " + nombre + ", ¿cómo estai?"
}

// devolver pa' cortar antes
pega clasificar(nota) {
  cachai (nota < 4) {
    devolver "rojo, pa' la casa"
  }
  "azul, gané el ramo"
}

lorea(saludar("Tulio"))
lorea(clasificar(3.2))
lorea(clasificar(6.0))

// Funciones que usan otras funciones
pega promedio(a, b) {
  (a + b) / 2
}
lorea("Promedio: " + texto(promedio(5, 7)))