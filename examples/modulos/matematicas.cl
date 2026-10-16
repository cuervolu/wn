// Módulo de ejemplo: solo lo marcado con `convida` se puede usar desde afuera.

convida duro PI = 3.14159

convida wea llamadas = 0

// Privada: solo existe dentro de este archivo.
pega validar(a, b) {
    devolver cachar(a) == "numero" y cachar(b) == "numero"
}

convida pega sumar(a, b) {
    llamadas = llamadas + 1
    cachai (no validar(a, b)) {
        devolver nada
    }
    devolver a + b
}
