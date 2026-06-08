<script lang="ts">
	import { resolve } from '$app/paths';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import type { Lesson } from '$lib/content';
	import manifest from '$lib/content';

	let { lesson }: { lesson: Lesson } = $props();

	const orderedLessons = [...manifest].sort((a, b) => a.order - b.order);

	const currentIndex = $derived(orderedLessons.findIndex(({ slug }) => slug === lesson.slug));
	const previousLesson = $derived(currentIndex > 0 ? orderedLessons[currentIndex - 1] : null);
	const nextLesson = $derived(
		currentIndex >= 0 && currentIndex < orderedLessons.length - 1
			? orderedLessons[currentIndex + 1]
			: null
	);
</script>

<article class="tour-lesson">
	<span class="tour-lesson__eyebrow">{String(lesson.order).padStart(2, '0')} · {lesson.section}</span>
	<h1 class="tour-lesson__title">{lesson.title}</h1>
	<hr class="tour-lesson__rule" />

	<div class="lesson-prose">
		<lesson.ContentComponent />
	</div>

	<nav class="tour-lesson__pager" aria-label="Navegación entre lecciones">
		{#if previousLesson}
			<a class="tour-pager" href={resolve('/[slug]', { slug: previousLesson.slug })}>
				<span class="tour-pager__dir"><ArrowLeft size={13} /> Anterior</span>
				<span class="tour-pager__name">{previousLesson.title}</span>
			</a>
		{:else}
			<span class="tour-lesson__pager-spacer"></span>
		{/if}

		{#if nextLesson}
			<a class="tour-pager is-next" href={resolve('/[slug]', { slug: nextLesson.slug })}>
				<span class="tour-pager__dir">Siguiente <ArrowRight size={13} /></span>
				<span class="tour-pager__name">{nextLesson.title}</span>
			</a>
		{/if}
	</nav>
</article>
