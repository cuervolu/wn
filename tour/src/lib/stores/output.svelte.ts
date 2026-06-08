import type { ResultadoEjecucion } from '$lib/wasm';

function createOutputStore() {
	let result = $state<ResultadoEjecucion | null>(null);
	let loading = $state(false);

	return {
		get result() {
			return result;
		},
		get loading() {
			return loading;
		},
		set(r: ResultadoEjecucion) {
			result = r;
			loading = false;
		},
		reset() {
			result = null;
			loading = false;
		},
		clear() {
			result = null;
			loading = true;
		}
	};
}

export const outputStore = createOutputStore();
