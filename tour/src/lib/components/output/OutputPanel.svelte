<script lang="ts">
	import Play from '@lucide/svelte/icons/play';
	import { outputStore } from '$lib/stores/output.svelte';
</script>

<div class="flex h-full flex-col bg-surface-950 font-mono text-sm">
	<div class="shrink-0 border-b border-surface-800 bg-surface-900 px-4 py-1.5">
		<span class="text-xs text-surface-500">output</span>
	</div>

	<div class="flex-1 overflow-y-auto p-4">
		{#if outputStore.loading}
			<p class="animate-pulse text-xs text-surface-500">ejecutando...</p>

		{:else if outputStore.result === null}
			<!-- Hint de estado vacío — usa el mismo icono que el botón -->
			<p class="flex items-center gap-1.5 text-xs text-surface-600">
				Presiona
				<kbd class="inline-flex items-center gap-1 rounded bg-surface-800 px-1.5 py-0.5 text-surface-400">
					<Play size={10} />
					ejecutar
				</kbd>
				para ver el output.
			</p>

		{:else if outputStore.result.error}
			{@const diag = outputStore.result.error}
			<div class="space-y-3">
				{#if outputStore.result.salida}
					<pre class="whitespace-pre-wrap text-xs text-surface-200">{outputStore.result.salida}</pre>
					<hr class="border-surface-800" />
				{/if}

				<div class="rounded border border-error-500/30 bg-error-500/10 p-3">
					<div class="mb-2 flex items-center gap-2">
						<span class="rounded bg-error-500/20 px-1.5 py-0.5 text-xs font-semibold text-error-400">
							{diag.fase}
						</span>
						{#if diag.linea !== undefined}
							<span class="text-xs text-surface-500">
								línea {diag.linea}
								{#if diag.offset !== undefined}· offset {diag.offset}{/if}
							</span>
						{/if}
					</div>
					<pre class="whitespace-pre-wrap text-xs text-error-300">{diag.mensaje}</pre>
				</div>
			</div>

		{:else if outputStore.result.salida === ''}
			<p class="text-xs italic text-surface-600">(sin output)</p>

		{:else}
			<pre class="whitespace-pre-wrap text-xs text-surface-100">{outputStore.result.salida}</pre>
		{/if}
	</div>
</div>