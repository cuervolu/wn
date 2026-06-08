<script lang="ts">
	import Play from '@lucide/svelte/icons/play';
	import { outputStore } from '$lib/stores/output.svelte';

	function outputLines(salida: string): string[] {
		if (salida === '') return [];
		const normalized = salida.endsWith('\n') ? salida.slice(0, -1) : salida;
		return normalized.split('\n');
	}

	const result = $derived(outputStore.result);
	const lines = $derived(result ? outputLines(result.salida) : []);
	const hasError = $derived(!!result?.error);
</script>

<div class="output-panel">
	<div class="output-panel__head">
		<span class:output-panel__dot={true} class:is-ok={!!result && !hasError} class:is-err={hasError}></span>
		<span class="output-panel__title">output</span>
		{#if result}
			<span class="output-panel__meta">
				{#if hasError}
					con error
				{:else}
					{lines.length} {lines.length === 1 ? 'línea' : 'líneas'}
				{/if}
			</span>
		{/if}
	</div>

	<div class="output-panel__body tour-scroll">
		{#if outputStore.loading}
			<p class="output-panel__empty">ejecutando...</p>
		{:else if result === null}
			<p class="output-panel__empty">
				Apretá
				<span class="output-panel__empty-key">
					<Play size={12} />
					ejecutar
				</span>
				pa' ver el output.
			</p>
		{:else if lines.length === 0 && !hasError}
			<p class="output-panel__empty">El programa corrió sin imprimir nada.</p>
		{:else}
			{#each lines as line}
				<div class="output-panel__line">
					<span class="output-panel__mark">›</span>
					<span>{line}</span>
				</div>
			{/each}

			{#if result?.error}
				{@const diag = result.error}
				<div class="output-panel__error">
					<div class="output-panel__error-top">
						<span class="output-panel__error-kind">{diag.fase}</span>
						<span class="output-panel__error-loc">
							{#if diag.linea !== undefined}
								línea {diag.linea}
								{#if diag.offset !== undefined} · offset {diag.offset}{/if}
							{:else}
								sin ubicación
							{/if}
						</span>
					</div>
					<div class="output-panel__error-msg">{diag.mensaje}</div>
				</div>
			{/if}
		{/if}
	</div>
</div>
