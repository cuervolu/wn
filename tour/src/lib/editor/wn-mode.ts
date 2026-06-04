import { StreamLanguage, HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags } from '@lezer/highlight';
import type { StreamParser } from '@codemirror/language';

interface WnState {
	inString: boolean;
	stringChar: string;
}

const wnParser: StreamParser<WnState> = {
	name: 'wn',

	startState(): WnState {
		return { inString: false, stringChar: '' };
	},

	token(stream, state) {
		if (state.inString) {
			while (!stream.eol()) {
				const ch = stream.next();
				if (ch === '\\') {
					stream.next();
				} else if (ch === state.stringChar) {
					state.inString = false;
					break;
				}
			}
			return 'string';
		}

		if (stream.match('//')) {
			stream.skipToEnd();
			return 'comment';
		}

		if (stream.peek() === '$') {
			const next = stream.string[stream.pos + 1];
			if (next === '"' || next === "'") {
				stream.next();
				state.inString = true;
				state.stringChar = stream.next()!;
				return 'string';
			}
		}

		if (stream.peek() === '"' || stream.peek() === "'") {
			state.inString = true;
			state.stringChar = stream.next()!;
			return 'string';
		}

		if (stream.match(/^[0-9]+(\.[0-9]+)?/)) {
			return 'number';
		}

		if (stream.match(/^[a-zA-Z_][a-zA-Z0-9_]*/)) {
			const word = stream.current();

			const keywords = new Set([
				'wea',
				'duro',
				'pega',
				'cachai',
				'si',
				'no',
				'mientras',
				'para',
				'en',
				'ojo',
				'cago',
				'devolver',
				'cortala',
				'sigue',
				'según'
			]);
			const builtins = new Set(['lorea', 'largo', 'cachar', 'pregunta', 'numero', 'texto']);
			const literals = new Set(['verdad', 'falso', 'nada']);

			if (keywords.has(word)) return 'keyword';
			if (builtins.has(word)) return 'builtin';
			if (literals.has(word)) return 'atom';
			return 'variableName';
		}

		if (stream.match(/^(==|!=|<=|>=|->|[+\-*/%=<>!])/)) {
			return 'operator';
		}

		stream.next();
		return null;
	},

	languageData: {
		commentTokens: { line: '//' }
	}
};

export const wnLanguage = StreamLanguage.define(wnParser);

export const wnHighlight = syntaxHighlighting(
	HighlightStyle.define([
		{ tag: tags.keyword, fontWeight: '700' },
		{ tag: tags.standard(tags.name), fontWeight: '600' },
		{ tag: tags.atom, fontWeight: '600' }
	])
);
