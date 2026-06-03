<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view';
	import { EditorState } from '@codemirror/state';
	import { defaultKeymap, historyKeymap, history } from '@codemirror/commands';
	import { oneDark } from '@codemirror/theme-one-dark';
	import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
	import Terminal from '@lucide/svelte/icons/terminal';
	import { wnLanguage, wnHighlight } from '$lib/editor/wn-mode';
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
					// Aplicar en todos los selectores donde CM renderiza texto —
					// oneDark sobreescribe fontFamily en .cm-content
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

	async function run() {
		if (running) return;
		running = true;
		outputStore.clear();
		const result = await ejecutarWasm(lessonStore.userCode, lessonStore.stdin);
		outputStore.set(result);
		running = false;
	}
</script>

<div class="flex h-full flex-col bg-surface-950">
	<div
		class="flex items-center justify-between border-b border-surface-800 bg-surface-900 px-3 py-1.5"
	>
		<div class="flex items-center gap-3">
			<button
				onclick={lessonStore.resetCode}
				class="flex items-center gap-1 font-mono text-xs text-surface-500 transition-colors hover:text-surface-300"
				title="Restaurar código inicial"
			>
				<RotateCcw size={12} />
				reset
			</button>
			<button
				onclick={lessonStore.toggleStdin}
				class="flex items-center gap-1 font-mono text-xs transition-colors
					{lessonStore.stdinOpen ? 'text-primary-400' : 'text-surface-500 hover:text-surface-300'}"
				title="Entrada para pregunta()"
			>
				<Terminal size={12} />
				stdin
			</button>
		</div>
		<RunButton {running} onclick={run} />
	</div>

	{#if lessonStore.stdinOpen}
		<StdinDrawer />
	{/if}

	<div bind:this={editorContainer} class="min-h-0 flex-1 overflow-hidden"></div>
</div>
