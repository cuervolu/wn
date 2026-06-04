ojo {
  wea resultado = 10 / 0
  lorea(resultado)
} cago(error) {
  lorea("Atrapado: " + error)
}

ojo {
  lorea(variable_fantasma)
} cago(e) {
  lorea("También atrapado: " + e)
}

//Intenta romper esto — cambia el índice
wea lista = ["uno", "dos", "tres"]
ojo {
  lorea(lista[10])
} cago(e) {
  lorea(e)
}