<script lang="ts">
	import { redirect } from '@sveltejs/kit';
	import { resolve } from '$app/paths';
	import manifest from '$lib/content';

	const first = manifest[0];
	if (first) {
		redirect(307, resolve('/[slug]', { slug: first.slug }));
	}
</script>

<div class="flex min-h-screen items-center justify-center">
	<p class="font-mono text-surface-400">No hay lecciones disponibles.</p>
</div>