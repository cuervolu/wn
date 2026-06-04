<script lang="ts">
	import type { Lesson } from '$lib/content';
	import Sidebar from './Sidebar.svelte';
	import MobileNav from './MobileNav.svelte';
	import LessonContent from '../lesson/LessonContent.svelte';
	import Editor from '../editor/Editor.svelte';
	import OutputPanel from '../output/OutputPanel.svelte';
	import ThemeToggle from '../ui/ThemeToggle.svelte';
	import { PaneGroup, Pane, PaneResizer } from 'paneforge';

	let { lesson }: { lesson: Lesson } = $props();

	type MobileTab = 'lesson' | 'editor';
	let mobileTab = $state<MobileTab>('lesson');
</script>

<div class="hidden h-screen w-full overflow-hidden bg-surface-950 md:flex">
	<aside
		class="shrink-0 overflow-y-auto border-r border-surface-800"
		style="width: var(--sidebar-width)"
	>
		<Sidebar activeSlug={lesson.slug} />
	</aside>

	<PaneGroup direction="horizontal" class="h-full min-w-0 flex-1">

		<Pane defaultSize={38} minSize={20} class="border-r border-surface-800">
			<div class="h-full overflow-y-auto">
				<LessonContent {lesson} />
			</div>
		</Pane>

		<!-- Resizer horizontal -->
		<PaneResizer
			class="
				group relative w-1.5 shrink-0
				cursor-col-resize bg-surface-800
				transition-colors duration-150
				hover:bg-primary-500/50 active:bg-primary-500
			"
		>
			<div class="
				absolute top-1/2 left-1/2
				flex -translate-x-1/2 -translate-y-1/2
				flex-col gap-1
				opacity-0 transition-opacity group-hover:opacity-100
			">
				<span class="block h-1 w-1 rounded-full bg-surface-300"></span>
				<span class="block h-1 w-1 rounded-full bg-surface-300"></span>
				<span class="block h-1 w-1 rounded-full bg-surface-300"></span>
			</div>
		</PaneResizer>

		<Pane minSize={30} class="flex min-w-0 flex-col">
			<div class="flex items-center justify-between border-b border-surface-800 bg-surface-900 px-4 py-2">
				<span class="font-mono text-xs text-surface-400">playground.cl</span>
				<ThemeToggle />
			</div>

			<!-- PaneGroup vertical ocupa el espacio bajo el header -->
			<PaneGroup direction="vertical" class="min-h-0 flex-1">

				<!-- Editor -->
				<Pane defaultSize={70} minSize={20}>
					<div class="h-full overflow-hidden">
						<Editor />
					</div>
				</Pane>

				<!-- Resizer vertical -->
				<PaneResizer
					class="
						group relative h-1.5 shrink-0
						cursor-row-resize bg-surface-800
						transition-colors duration-150
						hover:bg-primary-500/50 active:bg-primary-500
					"
				>
					<div class="
						absolute top-1/2 left-1/2
						flex -translate-x-1/2 -translate-y-1/2
						flex-row gap-1
						opacity-0 transition-opacity group-hover:opacity-100
					">
						<span class="block h-1 w-1 rounded-full bg-surface-300"></span>
						<span class="block h-1 w-1 rounded-full bg-surface-300"></span>
						<span class="block h-1 w-1 rounded-full bg-surface-300"></span>
					</div>
				</PaneResizer>

				<!-- Output -->
				<Pane defaultSize={30} minSize={10}>
					<div class="h-full overflow-hidden border-t border-surface-800">
						<OutputPanel />
					</div>
				</Pane>

			</PaneGroup>
		</Pane>

	</PaneGroup>
</div>

<div class="flex min-h-dvh w-full flex-col bg-surface-950 md:hidden">
	<header class="shrink-0 border-b border-surface-800 bg-surface-900">
		<div class="flex items-center justify-between px-4 py-2">
			<span class="font-mono text-sm font-bold text-surface-50">WN++ Tour</span>
			<ThemeToggle />
		</div>
		<MobileNav bind:activeTab={mobileTab} title={lesson.title} />
	</header>

	<main class="min-h-0 flex-1 overflow-y-auto">
		{#if mobileTab === 'lesson'}
			<div class="min-h-full">
				<LessonContent {lesson} />
			</div>
		{:else}
			<div class="flex h-full min-h-0 flex-col overflow-hidden">
				<div class="min-h-0 flex-1">
					<Editor />
				</div>
				<div class="h-40 border-t border-surface-800">
					<OutputPanel />
				</div>
			</div>
		{/if}
	</main>
</div>
