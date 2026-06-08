<script lang="ts">
	import { lessonStore } from '$lib/stores/lesson.svelte';
</script>

<div class="editor-panel__stdin">
	<label for="stdin-input" class="editor-panel__stdin-label">
		<span class="editor-panel__stdin-label-dot"></span>
		stdin — una línea por llamada a pregunta()
	</label>
	<textarea
		id="stdin-input"
		bind:value={lessonStore.stdin}
		rows={3}
		placeholder={'línea 1\nlínea 2\n...'}
		class="editor-panel__stdin-textarea"
	></textarea>
</div>
