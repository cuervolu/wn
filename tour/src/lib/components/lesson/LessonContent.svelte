<script lang="ts">
	import type { Lesson } from '$lib/content';

	let { lesson }: { lesson: Lesson } = $props();
</script>

<article class="px-6 py-8">
	<div class="mb-6 flex items-baseline gap-3">
		<span class="font-mono text-sm text-surface-600">
			{String(lesson.order).padStart(2, '0')}
		</span>
		<h1 class="m-0 text-2xl font-bold text-surface-50">{lesson.title}</h1>
	</div>

	<div class="lesson-prose prose max-w-none prose-invert">
		<lesson.ContentComponent />
	</div>
</article>
