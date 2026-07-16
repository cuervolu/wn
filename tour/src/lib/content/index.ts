import type { SvelteComponent } from 'svelte';

export interface LessonMeta {
	slug: string;
	title: string;
	section: string;
	order: number;
	dir: string;
}

export interface Lesson extends LessonMeta {
	ContentComponent: typeof SvelteComponent;
	initialCode: string;
}

export interface Section {
	title: string;
	lessons: LessonMeta[];
}

const DIR_PATTERN = /\/lessons\/(\d+)-([a-z0-9-]+)\/content\.md$/;
const FRONTMATTER_PATTERN = /^---\r?\n([\s\S]*?)\r?\n---/;

function parseFrontmatter(raw: string, path: string): { title: string; section: string } {
	const match = raw.match(FRONTMATTER_PATTERN);
	if (!match) throw new Error(`Falta frontmatter en ${path}`);

	const data: Record<string, string> = {};
	for (const line of match[1].split(/\r?\n/)) {
		const kv = line.match(/^(\w+):\s*["']?(.*?)["']?\s*$/);
		if (kv) data[kv[1]] = kv[2];
	}

	if (!data.title || !data.section) {
		throw new Error(`Frontmatter incompleto (title/section) en ${path}`);
	}
	return { title: data.title, section: data.section };
}

const rawContent = import.meta.glob<string>('/src/content/lessons/*/content.md', {
	eager: true,
	query: '?raw',
	import: 'default'
});

const manifest: LessonMeta[] = Object.entries(rawContent)
	.map(([path, raw]) => {
		const m = path.match(DIR_PATTERN);
		if (!m) throw new Error(`Carpeta de lección con nombre inválido: ${path}`);
		const [, orderStr, slug] = m;
		const { title, section } = parseFrontmatter(raw, path);
		return { slug, title, section, order: Number(orderStr), dir: `${orderStr}-${slug}` };
	})
	.sort((a, b) => a.order - b.order);

const seenSlugs = new Set<string>();
const seenOrders = new Set<number>();
for (const l of manifest) {
	if (seenSlugs.has(l.slug)) throw new Error(`Slug duplicado: ${l.slug}`);
	if (seenOrders.has(l.order)) throw new Error(`Order duplicado: ${l.order}`);
	seenSlugs.add(l.slug);
	seenOrders.add(l.order);
}

export default manifest;

export function groupBySections(lessons: LessonMeta[]): Section[] {
	const map = new Map<string, LessonMeta[]>();
	for (const lesson of lessons) {
		if (!map.has(lesson.section)) map.set(lesson.section, []);
		map.get(lesson.section)!.push(lesson);
	}
	return Array.from(map.entries()).map(([title, items]) => ({
		title,
		lessons: items.sort((a, b) => a.order - b.order)
	}));
}
