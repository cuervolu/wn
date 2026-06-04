// Calculadora de boleta chilena
duro IVA = 0.19

pega neto(bruto) {
  bruto / (1 + IVA)
}

pega impuesto(bruto) {
  bruto - neto(bruto)
}

pega resumen(nombre, bruto) {
  lorea("--- " + nombre + " ---")
  lorea("Bruto:    $" + texto(bruto))
  lorea("Neto:     $" + texto(neto(bruto)))
  lorea("IVA:      $" + texto(impuesto(bruto)))
}

resumen("Computador", 599990)
resumen("Teclado", 49990)
