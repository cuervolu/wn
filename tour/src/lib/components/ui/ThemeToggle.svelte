<script lang="ts">
	import Moon from '@lucide/svelte/icons/moon';
	import Sun from '@lucide/svelte/icons/sun';
	import { themeStore } from '$lib/stores/theme.svelte';
</script>

<div class="theme-toggle" aria-label="Cambiar tema">
	<button
		type="button"
		class:theme-toggle__button={true}
		class:is-on={!themeStore.isDark}
		title="Tema claro"
		aria-label="Tema claro"
		onclick={() => themeStore.set('mona')}
	>
		<Sun />
	</button>
	<button
		type="button"
		class:theme-toggle__button={true}
		class:is-on={themeStore.isDark}
		title="Tema oscuro"
		aria-label="Tema oscuro"
		onclick={() => themeStore.set('cerberus')}
	>
		<Moon />
	</button>
</div>
