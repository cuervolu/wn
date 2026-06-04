<script lang="ts">
	let {
		running,
		onclick
	}: {
		running: boolean;
		onclick: () => void;
	} = $props();
</script>

<button
	{onclick}
	disabled={running}
	class="
		flex items-center gap-1.5 px-3 py-1 rounded text-xs font-mono font-medium
		transition-all
		{running
			? 'bg-surface-700 text-surface-500 cursor-not-allowed'
			: 'bg-primary-500 hover:bg-primary-400 text-surface-950 cursor-pointer'}
	"
>
	{#if running}
		<span class="animate-spin text-xs">◌</span>
		ejecutando...
	{:else}
		▶ ejecutar
	{/if}
</button>