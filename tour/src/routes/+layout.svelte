<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import '@fontsource-variable/space-grotesk/wght.css';
	import '@fontsource-variable/jetbrains-mono/wght.css';
	import { themeStore } from '$lib/stores/theme.svelte';
	$effect(() => {
		document.documentElement.setAttribute('data-theme', themeStore.current);
	});
	let { children } = $props();
</script>

<svelte:head>
	<script>
		const saved = localStorage.getItem('wn-theme');
		if (saved) document.documentElement.setAttribute('data-theme', saved);
	</script>
	<link rel="icon" type="image/svg+xml" href={favicon} />
	<meta name="description" content="Tour interactivo de WN++ — aprende el lenguaje chileno en tu browser." />
	<title>Tour · WN++</title>
</svelte:head>

{@render children()}
