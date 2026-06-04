// Discografía de Los Prisioneros
wea discos = [
  {"titulo": "La voz de los '80", "año": 1984, "canciones": 10},
  {"titulo": "Pateando piedras",  "año": 1986, "canciones": 11},
  {"titulo": "La cultura de la basura", "año": 1987, "canciones": 11}
]

lorea("Discografía: " + texto(largo(discos)) + " álbumes")
lorea("")

wea i = 0
mientras (i < largo(discos)) {
  wea disco = discos[i]
  lorea(disco["titulo"] + " (" + texto(disco["año"]) + ")")
  i = i + 1
}

// Agrega un campo nuevo al último disco
discos[-1]["destacada"] = "We Are Sudamerican Rockers"
lorea("")
lorea("Canción destacada del último álbum: " + discos[-1]["destacada"])
