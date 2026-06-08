<script lang="ts">
	type Tab = 'lesson' | 'editor';

	let {
		activeTab = $bindable<Tab>('lesson')
	}: {
		activeTab: Tab;
	} = $props();
</script>

<div class="mobile-nav">
	<button
		type="button"
		onclick={() => (activeTab = 'lesson')}
		class:mobile-nav__button={true}
		class:is-active={activeTab === 'lesson'}
	>
		Lección
	</button>
	<button
		type="button"
		onclick={() => (activeTab = 'editor')}
		class:mobile-nav__button={true}
		class:is-active={activeTab === 'editor'}
	>
		Editor
	</button>
</div>
