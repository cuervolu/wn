import type { CodeOutputLine } from './content';
import { WnCodeBlock } from './wn-code-block';

type CodeWindowProps = {
  filename: string;
  code: string;
  output?: CodeOutputLine[];
  compact?: boolean;
};

export function CodeWindow({ filename, code, output = [], compact = false }: CodeWindowProps) {
  return (
    <div className="wn-code-window">
      <div className="wn-code-window__bar">
        <span className="wn-code-window__dots" aria-hidden="true">
          <i />
          <i />
          <i />
        </span>
        <span className="wn-code-window__name">{filename}</span>
        <span className="wn-code-window__tag">.cl</span>
      </div>

      <div className="wn-code-window__body">
        <WnCodeBlock code={code} compact={compact} />
      </div>

      {output.length > 0 ? (
        <div className="wn-code-window__output">
          <div className="wn-code-window__output-head">
            <span className="wn-code-window__output-dot" />
            salida
          </div>

          <div className="wn-code-window__output-body">
            {output.map((line, index) => (
              <div key={`${line.kind}-${index}`} className={`wn-output-line wn-output-line--${line.kind}`}>
                <span className="wn-output-line__mark">{line.kind === 'err' ? '✗' : '›'}</span>
                <span className="wn-output-line__text">{line.text}</span>
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}
