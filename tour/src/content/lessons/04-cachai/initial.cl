// Sistema de notas universitarias chileno
wea nota = 45

cachai (nota >= 70) {
  lorea("Aprobado con distinción")
} si no {
  cachai (nota >= 55) {
    lorea("Aprobado")
  } si no {
    cachai (nota >= 45) {
      lorea("Reprobado, pero cerca — puedes ir a examen")
    } si no {
      lorea("Reprobado")
    }
  }
}

// Prueba cambiando la nota: 30, 50, 60, 75
