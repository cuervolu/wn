// mientras: repite hasta que se acabe la wea
wea cervezas = 6
mientras (cervezas > 0) {
  lorea("Quedan " + texto(cervezas) + " chelas")
  cervezas = cervezas - 1
}
lorea("Se acabaron, hay que ir al negocio.")

// para...en: recorre una lista
wea completos = ["italiano","cualquiera, casera", "a lo pobre"]
para (c en completos) {
  cachai (c == "cualquiera, casera") {
    lorea("Un completo " + c + ", me importa un pico")
    sigue
  }
  lorea("Un completo " + c + ", por favor")
}
