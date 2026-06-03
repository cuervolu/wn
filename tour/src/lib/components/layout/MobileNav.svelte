<script lang="ts">
	type Tab = 'lesson' | 'editor';

	let {
		activeTab = $bindable<Tab>('lesson'),
		title
	}: {
		activeTab: Tab;
		title: string;
	} = $props();
</script>

<div class="flex border-b border-surface-700">
	<button
		onclick={() => (activeTab = 'lesson')}
		class="
			flex-1 border-b-2 py-2 text-sm font-medium transition-colors
			{activeTab === 'lesson'
			? 'border-primary-400 text-primary-300'
			: 'border-transparent text-surface-400 hover:text-surface-200'}
		"
	>
		Lección
	</button>
	<button
		onclick={() => (activeTab = 'editor')}
		class="
			flex-1 border-b-2 py-2 text-sm font-medium transition-colors
			{activeTab === 'editor'
			? 'border-primary-400 text-primary-300'
			: 'border-transparent text-surface-400 hover:text-surface-200'}
		"
	>
		Editor
	</button>
</div>
