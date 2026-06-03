<script lang="ts">
	import { lessonStore } from '$lib/stores/lesson.svelte';
</script>

<div class="border-b border-surface-700 bg-surface-900 px-3 py-2">
	<label for="stdin-input" class="mb-1 block font-mono text-xs text-surface-500">
		entrada estándar (stdin) — una línea por llamada a pregunta()
	</label>
	<textarea
		id="stdin-input"
		bind:value={lessonStore.stdin}
		rows={3}
		placeholder={'línea 1\nlínea 2\n...'}
		class="w-full resize-none rounded border border-surface-700 bg-surface-800
			px-2 py-1.5 font-mono text-xs text-surface-100 transition-colors
			outline-none placeholder:text-surface-600 focus:border-primary-500"
	></textarea>
</div>
