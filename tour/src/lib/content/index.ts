import type { SvelteComponent } from 'svelte';
import manifestJson from '../../content/lessons/manifest.json';

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

const manifest: LessonMeta[] = manifestJson as LessonMeta[];
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
