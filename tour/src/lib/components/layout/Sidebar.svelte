<script lang="ts">
	import { resolve } from '$app/paths';
	import ExternalLink from '@lucide/svelte/icons/external-link';
	import manifest, { groupBySections } from '$lib/content';

	let { activeSlug }: { activeSlug: string } = $props();

	const sections = groupBySections(manifest);
</script>

<nav class="flex h-full flex-col py-4">
	<div class="mb-6 px-4">
		<a href={resolve('/')} class="group flex items-center gap-2">
			<span class="font-mono text-lg font-bold text-primary-400">WN++</span>
			<span class="text-sm text-surface-500 transition-colors group-hover:text-surface-300">
				tour
			</span>
		</a>
	</div>

	<!-- ToC -->
	<div class="flex-1 space-y-4 overflow-y-auto px-2">
		{#each sections as section}
			<div>
				<p class="mb-1 px-2 text-xs font-semibold tracking-widest text-surface-500 uppercase">
					{section.title}
				</p>
				<ul class="space-y-0.5">
					{#each section.lessons as lesson}
						{@const isActive = lesson.slug === activeSlug}
						<li>
							<a
								href={resolve('/[slug]', { slug: lesson.slug })}
								class="
									flex items-center gap-2 rounded px-2 py-1.5 text-sm transition-colors
									{isActive
										? 'bg-primary-500/20 font-medium text-primary-300'
										: 'text-surface-300 hover:bg-surface-800 hover:text-surface-50'}
								"
							>
								<span
									class="h-1.5 w-1.5 shrink-0 rounded-full
										{isActive ? 'bg-primary-400' : 'bg-transparent'}"
								></span>
								{lesson.title}
							</a>
						</li>
					{/each}
				</ul>
			</div>
		{/each}
	</div>

	<div class="mt-4 border-t border-surface-800 px-4 pt-4">
		<a
			href="https://github.com/cuervolu/wn"
			target="_blank"
			rel="noopener noreferrer"
			class="flex items-center gap-2 font-mono text-xs text-surface-500 transition-colors hover:text-surface-300"
		>
			<ExternalLink size={12} />
			cuervolu/wn
		</a>
	</div>
</nav>