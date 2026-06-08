'use client';

import { startTransition, useEffect, useState } from 'react';
import { useTheme } from 'next-themes';
import { createHighlighter } from 'shiki/bundle/web';
import wnGrammar from '../../../../grammars/wn.tmLanguage.json';

const highlighterPromise = createHighlighter({
  themes: ['github-light', 'one-dark-pro'],
  langs: [wnGrammar],
});

type WnCodeBlockProps = {
  code: string;
  compact?: boolean;
};

export function WnCodeBlock({ code, compact = false }: WnCodeBlockProps) {
  const { resolvedTheme } = useTheme();
  const [html, setHtml] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const theme = resolvedTheme === 'dark' ? 'one-dark-pro' : 'github-light';

    highlighterPromise
      .then((highlighter) =>
        highlighter.codeToHtml(code, {
          lang: 'wn',
          theme,
        }),
      )
      .then((result) => {
        if (cancelled) return;
        startTransition(() => {
          setHtml(result);
        });
      })
      .catch(() => {
        if (cancelled) return;
        startTransition(() => {
          setHtml(null);
        });
      });

    return () => {
      cancelled = true;
    };
  }, [code, resolvedTheme]);

  if (html) {
    return (
      <div
        className={compact ? 'wn-inline-codeblock is-compact' : 'wn-inline-codeblock'}
        dangerouslySetInnerHTML={{ __html: html }}
      />
    );
  }

  return (
    <div className={compact ? 'wn-inline-codeblock is-compact' : 'wn-inline-codeblock'}>
      <pre>
        <code>{code}</code>
      </pre>
    </div>
  );
}
