<script lang="ts">
	import type { Lesson } from '$lib/content';
	import { Pane, PaneGroup, PaneResizer } from 'paneforge';
	import Editor from '../editor/Editor.svelte';
	import LessonContent from '../lesson/LessonContent.svelte';
	import OutputPanel from '../output/OutputPanel.svelte';
	import ThemeToggle from '../ui/ThemeToggle.svelte';
	import MobileNav from './MobileNav.svelte';
	import Sidebar from './Sidebar.svelte';

	let { lesson }: { lesson: Lesson } = $props();

	type MobileTab = 'lesson' | 'editor';
	let mobileTab = $state<MobileTab>('lesson');
</script>

<div class="tour-app">
	<div class="tour-shell">
		<aside class="tour-sidebar-column">
			<Sidebar activeSlug={lesson.slug} />
		</aside>

		<PaneGroup direction="horizontal" class="tour-content-group">
			<Pane defaultSize={38} minSize={20} class="tour-lesson-pane">
				<div class="tour-lesson-pane__scroll tour-scroll">
					<LessonContent {lesson} />
				</div>
			</Pane>

			<PaneResizer class="tour-pane-resizer">
				<div class="tour-pane-resizer__handle" aria-hidden="true">
					<span></span>
					<span></span>
					<span></span>
				</div>
			</PaneResizer>

			<Pane minSize={30} class="tour-play-pane">
				<div class="tour-play-head">
					<span class="tour-play-head__dots" aria-hidden="true">
						<i style="background: var(--wn-rose)"></i>
						<i style="background: var(--wn-amber)"></i>
						<i style="background: var(--wn-blue)"></i>
					</span>
					<span class="tour-play-head__file">playground<span>.cl</span></span>
					<div class="tour-play-head__spacer"></div>
					<ThemeToggle />
				</div>

				<PaneGroup direction="vertical" class="tour-play-stack">
					<Pane defaultSize={70} minSize={20}>
						<div class="h-full overflow-hidden">
							<Editor />
						</div>
					</Pane>

					<PaneResizer class="tour-pane-resizer">
						<div class="tour-pane-resizer__handle" aria-hidden="true">
							<span></span>
							<span></span>
							<span></span>
						</div>
					</PaneResizer>

					<Pane defaultSize={30} minSize={10}>
						<div class="h-full overflow-hidden">
							<OutputPanel />
						</div>
					</Pane>
				</PaneGroup>
			</Pane>
		</PaneGroup>
	</div>

	<div class="tour-mobile">
		<header class="tour-mobile__header">
			<div class="tour-mobile__brandbar">
				<div class="tour-mobile__title">
					<strong>Wn<span class="tour-sidebar__brand-plus">++</span></strong>
					<span>{lesson.title}</span>
				</div>
				<div class="tour-play-head__spacer"></div>
				<ThemeToggle />
			</div>
			<MobileNav bind:activeTab={mobileTab} />
		</header>

		<main class="tour-mobile__main">
			{#if mobileTab === 'lesson'}
				<LessonContent {lesson} />
			{:else}
				<div class="tour-mobile__editor">
					<div class="min-h-0 flex-1">
						<Editor />
					</div>
					<div class="tour-mobile__output">
						<OutputPanel />
					</div>
				</div>
			{/if}
		</main>
	</div>
</div>
