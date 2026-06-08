import { mdsvex, escapeSvelte } from 'mdsvex';
import adapter from '@sveltejs/adapter-cloudflare';
import { createHighlighter } from 'shiki';
import wnGrammar from '../grammars/wn.tmLanguage.json' with { type: 'json' };

const highlighter = await createHighlighter({
	themes: ['one-dark-pro'],
	langs: ['javascript', 'typescript', 'rust', 'bash', 'json', wnGrammar]
});

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
		runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true)
	},
	kit: { adapter: adapter() },
	preprocess: [
		mdsvex({
			extensions: ['.svx', '.md'],
			highlight: {
				highlighter: async (code, lang = 'text') => {
					// Si el lang no está cargado (ej: "wn", "cl", "piola"), cae a texto plano
					const safeLang = highlighter.getLoadedLanguages().includes(lang) ? lang : 'text';

					const html = escapeSvelte(
						highlighter.codeToHtml(code, { lang: safeLang, theme: 'one-dark-pro' })
					);
					return `{@html \`${html}\`}`;
				}
			}
		})
	],
	extensions: ['.svelte', '.svx', '.md']
};

export default config;
