// Festival de música chileno
duro PRECIO_ENTRADA = 45000
duro DESCUENTO_ESTUDIANTE = 10000
wea asistentes = 8

wea precio_con_descuento = PRECIO_ENTRADA - DESCUENTO_ESTUDIANTE
wea recaudacion = precio_con_descuento * asistentes

lorea("Precio con descuento: " + texto(precio_con_descuento))
lorea("Recaudación total: " + texto(recaudacion))

// ¿Alcanza pa' pagar el local? (valor: 300000)
wea alcanza = recaudacion >= 300000
lorea("¿Alcanza pa' el local? " + texto(alcanza))

// ¿Es número impar?
wea numero = 7
lorea(texto(numero) + " es impar: " + texto(numero % 2 != 0))
