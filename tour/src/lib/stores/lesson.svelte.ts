import { untrack } from 'svelte';
import type { Lesson } from '$lib/content';

function createLessonStore() {
	let activeLesson = $state<Lesson | null>(null);
	let userCode = $state<string>('');
	let stdin = $state<string>('');
	let stdinOpen = $state<boolean>(false);

	function setLesson(lesson: Lesson) {
		// untrack() previene que estas *lecturas* subscriban al $effect llamador.
		// Sin untrack: el $effect en +page.svelte se suscribe a userCode y
		// activeLesson → setLesson los escribe → effect re-corre → loop infinito.
		const prevSlug = untrack(() => activeLesson?.slug);
		const currentCode = untrack(() => userCode);

		activeLesson = lesson;

		// Solo resetea el código si es una lección distinta o el editor está vacío
		if (currentCode === '' || prevSlug !== lesson.slug) {
			userCode = lesson.initialCode;
		}

		stdin = '';
		stdinOpen = false;
	}

	function resetCode() {
		if (activeLesson) {
			userCode = activeLesson.initialCode;
		}
	}

	return {
		get lesson() {
			return activeLesson;
		},
		get userCode() {
			return userCode;
		},
		set userCode(v: string) {
			userCode = v;
		},
		get stdin() {
			return stdin;
		},
		set stdin(v: string) {
			stdin = v;
		},
		get stdinOpen() {
			return stdinOpen;
		},
		toggleStdin() {
			stdinOpen = !stdinOpen;
		},
		setLesson,
		resetCode
	};
}

export const lessonStore = createLessonStore();
