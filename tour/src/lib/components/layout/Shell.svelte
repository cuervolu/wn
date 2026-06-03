<script lang="ts">
	import type { Lesson } from '$lib/content';
	import Sidebar from './Sidebar.svelte';
	import MobileNav from './MobileNav.svelte';
	import LessonContent from '../lesson/LessonContent.svelte';
	import Editor from '../editor/Editor.svelte';
	import OutputPanel from '../output/OutputPanel.svelte';
	import ThemeToggle from '../ui/ThemeToggle.svelte';

	let { lesson }: { lesson: Lesson } = $props();

	type MobileTab = 'lesson' | 'editor';
	let mobileTab = $state<MobileTab>('lesson');
</script>

<div class="hidden md:flex h-screen w-full overflow-hidden bg-surface-950">

	<aside
		class="shrink-0 border-r border-surface-800 overflow-y-auto"
		style="width: var(--sidebar-width)"
	>
		<Sidebar activeSlug={lesson.slug} />
	</aside>

	<section
		class="shrink-0 border-r border-surface-800 overflow-y-auto"
		style="width: var(--lesson-width)"
	>
		<LessonContent {lesson} />
	</section>

	<div class="flex flex-col flex-1 min-w-0">
		<div class="flex items-center justify-between px-4 py-2 border-b border-surface-800 bg-surface-900">
			<span class="text-surface-400 text-xs font-mono">playground.cl</span>
			<ThemeToggle />
		</div>

		<div class="flex-1 min-h-0 overflow-hidden">
			<Editor />
		</div>

		<div class="h-48 border-t border-surface-800 overflow-hidden">
			<OutputPanel />
		</div>
	</div>
</div>

<div class="flex flex-col h-screen w-full overflow-hidden bg-surface-950 md:hidden">

	<header class="shrink-0 border-b border-surface-800 bg-surface-900">
		<div class="flex items-center justify-between px-4 py-2">
			<span class="text-surface-50 font-bold text-sm font-mono">WN++ Tour</span>
			<ThemeToggle />
		</div>
		<MobileNav bind:activeTab={mobileTab} title={lesson.title} />
	</header>

	<main class="flex-1 min-h-0 overflow-hidden">
		{#if mobileTab === 'lesson'}
			<div class="h-full overflow-y-auto">
				<LessonContent {lesson} />
			</div>
		{:else}
			<div class="flex flex-col h-full">
				<div class="flex-1 min-h-0">
					<Editor />
				</div>
				<div class="h-40 border-t border-surface-800">
					<OutputPanel />
				</div>
			</div>
		{/if}
	</main>
</div>