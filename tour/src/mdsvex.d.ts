// Declaración ambient para archivos .md procesados por mdsvex.
// Fuente: https://mdsvex.pngwn.io/docs#extensions
declare module '*.md' {
	import type { SvelteComponent } from 'svelte';
	export default class Comp extends SvelteComponent {}
	export const metadata: Record<string, unknown>;
}
