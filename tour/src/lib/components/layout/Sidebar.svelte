<script lang="ts">
	import { resolve } from '$app/paths';
	import ExternalLink from '@lucide/svelte/icons/external-link';
	import manifest, { groupBySections } from '$lib/content';

	let { activeSlug }: { activeSlug: string } = $props();

	const sections = groupBySections(manifest);
</script>

<nav class="tour-sidebar">
	<div class="tour-sidebar__top">
		<a href={resolve('/')} class="tour-sidebar__brand">
			<span>Wn</span>
			<span class="tour-sidebar__brand-plus">++</span>
		</a>
		<span class="tour-sidebar__tag">tour</span>
	</div>

	<div class="tour-sidebar__nav tour-scroll">
		{#each sections as section}
			<section class="tour-sidebar__group">
				<p class="tour-sidebar__label">{section.title}</p>
				<ul class="tour-sidebar__list">
					{#each section.lessons as lesson}
						<li>
							<a
								href={resolve('/[slug]', { slug: lesson.slug })}
								class:tour-sidebar__item={true}
								class:is-active={lesson.slug === activeSlug}
							>
								<span class="tour-sidebar__num">{String(lesson.order).padStart(2, '0')}</span>
								{lesson.title}
							</a>
						</li>
					{/each}
				</ul>
			</section>
		{/each}
	</div>

	<div class="tour-sidebar__footer">
		<a
			href="https://github.com/cuervolu/wn"
			target="_blank"
			rel="noopener noreferrer"
			class="tour-sidebar__repo"
		>
			<ExternalLink size={12} />
			cuervolu/wn
		</a>
	</div>
</nav>
