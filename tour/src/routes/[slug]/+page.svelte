<script lang="ts">
	import type { PageData } from './$types';
	import Shell from '$lib/components/layout/Shell.svelte';
	import { lessonStore } from '$lib/stores/lesson.svelte';

	let { data }: { data: PageData } = $props();

	// Sincroniza la lección activa en el store global
	$effect(() => {
		lessonStore.setLesson(data.lesson);
	});
</script>

<svelte:head>
	<title>{data.lesson.title} · Tour WN++</title>
</svelte:head>

<Shell lesson={data.lesson} />