import logoSrc from '@/assets/logo.png';

export { logoSrc };

export type CodeOutputLine = {
  kind: 'out' | 'ok' | 'err';
  text: string;
};

export type Example = {
  id: string;
  label: string;
  blurb: string;
  filename: string;
  code: string;
  output: CodeOutputLine[];
};

export type Keyword = {
  kw: string;
  en: string;
};

export type ErrorCard = {
  code: string;
  when: string;
  msg: string;
};

export const heroCode = `// wena, mundo
pega saludar(nombre) {
  "wena, " + nombre + ", wn!"
}

wea quien = pregunta("¿Como te llamai? ")
lorea(saludar(quien))`;

export const heroOutput: CodeOutputLine[] = [
  { kind: 'out', text: '¿Como te llamai? Ángel' },
  { kind: 'ok', text: 'wena, Ángel, wn!' },
];

export const examples: Example[] = [
  {
    id: 'hola',
    label: 'Hola mundo',
    blurb:
      'Funciones con retorno implícito y stdin. Lo último que evalúa una función es lo que devuelve.',
    filename: 'wena.cl',
    code: `pega saludar(nombre) {
  cachai (largo(nombre) == 0) {
    devolver "wena, anónimo, wn!"
  }
  "wena, " + nombre + ", wn!"
}

lorea(saludar("Wn++"))
lorea(saludar(""))`,
    output: [
      { kind: 'ok', text: 'wena, Wn++, wn!' },
      { kind: 'ok', text: 'wena, anónimo, wn!' },
    ],
  },
  {
    id: 'fizz',
    label: 'FizzBuzz',
    blurb:
      'Condicionales encadenadas con cachai / si no, un while a la chilena y módulo.',
    filename: 'fizzbuzz.cl',
    code: `wea n = 1
mientras (n <= 15) {
  cachai (n % 15 == 0) {
    lorea("FizzBuzz")
  } si no cachai (n % 3 == 0) {
    lorea("Fizz")
  } si no cachai (n % 5 == 0) {
    lorea("Buzz")
  } si no {
    lorea(n)
  }
  n = n + 1
}`,
    output: [
      { kind: 'out', text: '1   2   Fizz   4   Buzz' },
      { kind: 'out', text: 'Fizz   7   8   Fizz   Buzz' },
      { kind: 'out', text: '11   Fizz   13   14   FizzBuzz' },
    ],
  },
  {
    id: 'tipos',
    label: 'Tipos',
    blurb:
      "Seis tipos, tipado débil. Listas con [], mapas con {}, y cachar() pa' preguntar de qué color es la wea.",
    filename: 'tipos.cl',
    code: `wea n = 42
wea s = "hola"
wea b = verdad
wea x = nada
wea xs = [1, 2, 3]
wea m = { lenguaje: "Wn++" }

lorea(cachar(xs))
lorea(largo(xs))
lorea(m.lenguaje)`,
    output: [
      { kind: 'out', text: 'lista' },
      { kind: 'out', text: '3' },
      { kind: 'out', text: 'Wn++' },
    ],
  },
  {
    id: 'error',
    label: 'Errores',
    blurb:
      'Acá está la gracia: cuando algo se cae, el intérprete te rezonga en chileno. Con cariño, eso sí.',
    filename: 'la_caga.cl',
    code: `ojo {
  wea total = 10 + "veinte"
} cago(e) {
  lorea(e)
}

wea r = 10 / 0`,
    output: [
      {
        kind: 'err',
        text: "No podi sumar un 'numero' con un 'texto', pedazo de saco wea.",
      },
      { kind: 'err', text: 'SUSPENSIÓN PERMANENTE: división por cero.' },
    ],
  },
];

export const keywords: Keyword[] = [
  { kw: 'wea', en: 'variable mutable' },
  { kw: 'duro', en: 'constante' },
  { kw: 'pega', en: 'función' },
  { kw: 'cachai', en: 'if' },
  { kw: 'si no', en: 'else' },
  { kw: 'mientras', en: 'while' },
  { kw: 'para … en', en: 'for-each' },
  { kw: 'ojo / cago', en: 'try / catch' },
  { kw: 'devolver', en: 'return' },
  { kw: 'cortala', en: 'break' },
  { kw: 'sigue', en: 'continue' },
  { kw: 'y · o · no', en: 'and · or · not' },
];

export const errors: ErrorCard[] = [
  {
    code: 'E_SACO',
    when: 'sumaste un número con un texto',
    msg: "No podi sumar un 'numero' con un 'texto', pedazo de saco wea.",
  },
  {
    code: 'E_NOEXISTE',
    when: 'usaste una variable que no declaraste',
    msg: "La wea 'x' no existe papito.",
  },
  {
    code: 'E_DIV0',
    when: 'dividiste por cero',
    msg: 'SUSPENSIÓN PERMANENTE: división por cero.',
  },
];

export const ribbonItems = [
  'WENA, WN!',
  'HECHO EN CHILE',
  'ESCRITO EN RUST',
  'ARCHIVOS .CL',
  'WEA X = WENA, WN!',
  'NO LLORÍ, PROGRAMÍ',
];
