<script lang="ts">
	import { EditorState } from '@codemirror/state';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
	import { oneDark } from '@codemirror/theme-one-dark';
	import { EditorView, highlightActiveLine, keymap, lineNumbers } from '@codemirror/view';
	import { onDestroy, onMount } from 'svelte';
	import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
	import Terminal from '@lucide/svelte/icons/terminal';
	import { wnHighlight, wnLanguage } from '$lib/editor/wn-mode';
	import { lessonStore } from '$lib/stores/lesson.svelte';
	import { outputStore } from '$lib/stores/output.svelte';
	import { ejecutarWasm } from '$lib/wasm';
	import RunButton from './RunButton.svelte';
	import StdinDrawer from './StdinDrawer.svelte';

	let editorContainer: HTMLDivElement;
	let view: EditorView | null = null;
	let running = $state(false);
	let syncingFromStore = false;

	onMount(() => {
		const startState = EditorState.create({
			doc: lessonStore.userCode,
			extensions: [
				history(),
				lineNumbers(),
				highlightActiveLine(),
				keymap.of([...defaultKeymap, ...historyKeymap]),
				wnLanguage,
				wnHighlight,
				oneDark,
				EditorView.updateListener.of((update) => {
					if (update.docChanged && !syncingFromStore) {
						lessonStore.userCode = update.state.doc.toString();
					}
				}),
				EditorView.theme({
					'&': { height: '100%', fontSize: '14px' },
					'.cm-content': {
						padding: '12px 0',
						fontFamily: "'JetBrains Mono Variable', 'JetBrains Mono', monospace"
					},
					'.cm-scroller': {
						overflow: 'auto',
						fontFamily: "'JetBrains Mono Variable', 'JetBrains Mono', monospace"
					},
					'.cm-gutters': {
						fontFamily: "'JetBrains Mono Variable', 'JetBrains Mono', monospace"
					}
				})
			]
		});

		view = new EditorView({ state: startState, parent: editorContainer });
	});

	$effect(() => {
		const incoming = lessonStore.userCode;
		if (!view) return;

		const current = view.state.doc.toString();
		if (current === incoming) return;

		syncingFromStore = true;
		view.dispatch({
			changes: { from: 0, to: current.length, insert: incoming }
		});
		syncingFromStore = false;
	});

	onDestroy(() => view?.destroy());

	function resetEditor() {
		lessonStore.resetCode();
		outputStore.reset();
	}

	async function run() {
		if (running) return;
		running = true;
		outputStore.clear();
		const result = await ejecutarWasm(lessonStore.userCode, lessonStore.stdin);
		outputStore.set(result);
		running = false;
	}
</script>

<div class="editor-panel">
	<div class="editor-panel__tools">
		<div class="flex items-center gap-2">
			<button
				type="button"
				onclick={resetEditor}
				class="editor-panel__tool"
				title="Restaurar código inicial"
			>
				<RotateCcw size={12} />
				reset
			</button>
			<button
				type="button"
				onclick={lessonStore.toggleStdin}
				class:editor-panel__tool={true}
				class:is-on={lessonStore.stdinOpen}
				title="Entrada para pregunta()"
			>
				<Terminal size={12} />
				stdin
			</button>
		</div>
		<div class="editor-panel__run">
			<RunButton {running} onclick={run} />
		</div>
	</div>

	{#if lessonStore.stdinOpen}
		<StdinDrawer />
	{/if}

	<div bind:this={editorContainer} class="editor-panel__editor"></div>
</div>
