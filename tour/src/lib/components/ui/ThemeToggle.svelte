<script lang="ts">
	import Sun from '@lucide/svelte/icons/sun';
	import Moon from '@lucide/svelte/icons/moon';
	import { themeStore } from '$lib/stores/theme.svelte';
</script>

<button
	onclick={themeStore.toggle}
	class="text-surface-500 hover:text-surface-200 transition-colors p-1 rounded"
	title="Cambiar tema ({themeStore.current})"
	aria-label="Toggle theme"
>
	{#if themeStore.isDark}
		<Sun size={16} />
	{:else}
		<Moon size={16} />
	{/if}
</button>