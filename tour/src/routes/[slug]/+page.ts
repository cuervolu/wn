import type { PageLoad } from './$types';
import type { SvelteComponent } from 'svelte';
import { error } from '@sveltejs/kit';
import manifest from '$lib/content';

export const prerender = true;

type MdsvexModule = {
	default: typeof SvelteComponent;
	metadata?: Record<string, unknown>;
};

export const load: PageLoad = async ({ params }) => {
	const meta = manifest.find((l) => l.slug === params.slug);
	if (!meta) error(404, `Lección '${params.slug}' no encontrada.`);

	const contentModules = import.meta.glob<MdsvexModule>('/src/content/lessons/*/content.md');
	const codeModules = import.meta.glob<string>('/src/content/lessons/*/initial.cl', {
		query: '?raw',
		import: 'default'
	});

	const contentPath = Object.keys(contentModules).find((p) =>
		p.includes(`/${meta.dir}/content.md`)
	);
	const codePath = Object.keys(codeModules).find((p) => p.includes(`/${meta.dir}/initial.cl`));

	if (!contentPath || !codePath) {
		error(500, `Archivos de la lección '${params.slug}' no encontrados.`);
	}

	const [contentMod, initialCode] = await Promise.all([
		contentModules[contentPath](),
		codeModules[codePath]()
	]);

	return {
		lesson: {
			...meta,
			ContentComponent: contentMod.default,
			initialCode
		}
	};
};