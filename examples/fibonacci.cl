pega fibonacci(n) {
  cachai (n <= 1) {
    n
  } si no {
    fibonacci(n - 1) + fibonacci(n - 2)
  }
}

lorea(fibonacci(10))