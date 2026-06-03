import init from './pkg/wn_wasm.js';
import { ejecutar as _ejecutar } from './pkg/wn_wasm.js';
import type { DiagnosticoWasm, ResultadoEjecucion } from './pkg/wn_wasm.js';

export type { DiagnosticoWasm, ResultadoEjecucion };

let initialized = false;
let initPromise: Promise<void> | null = null;

async function ensureInit(): Promise<void> {
  if (initialized) return;
  if (!initPromise) {
    initPromise = init().then(() => {
      initialized = true;
    });
  }
  return initPromise;
}

export async function ejecutarWasm(
    fuente: string,
    stdin: string = ''
): Promise<ResultadoEjecucion> {
  try {
    await ensureInit();
    return _ejecutar(fuente, stdin);
  } catch (err) {
    return {
      salida: '',
      error: {
        fase: 'carga',
        mensaje: `No se pudo cargar el motor WN++: ${err instanceof Error ? err.message : String(err)}`,
        len: undefined,
        linea: undefined,
        offset: undefined,
        free() {}
      } as DiagnosticoWasm,
      free() {}
    } as ResultadoEjecucion;
  }
}