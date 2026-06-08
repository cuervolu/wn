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
	type="button"
	{onclick}
	disabled={running}
	class:run-button={true}
	class:is-running={running}
>
	{#if running}
		<span class="animate-spin text-xs">◌</span>
		ejecutando...
	{:else}
		▶ ejecutar
	{/if}
</button>
