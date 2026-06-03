// Tabla de posiciones — Primera División chilena
wea equipos = ["Colo-Colo", "La U", "La Católica", "Huachipato", "Ñublense"]
wea puntos  = [45, 38, 36, 29, 27]

wea i = 0
mientras (i < largo(equipos)) {
  lorea(texto(i + 1) + ". " + equipos[i] + " — " + texto(puntos[i]) + " pts")
  i = i + 1
}

// ¿Cuántos equipos clasifican a Copa Libertadores? (top 3)
lorea("")
lorea("Clasifican a Libertadores:")
para (equipo en equipos) {
  wea pos = 0
  wea j = 0
  mientras (j < largo(equipos)) {
    cachai (equipos[j] == equipo) {
      pos = j
    }
    j = j + 1
  }
  cachai (pos < 3) {
    lorea("  " + equipo)
  }
}
