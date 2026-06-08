import { cn } from '@/lib/cn';

type SectionHeadingProps = {
  kicker: string;
  title: string;
  highlight?: string;
  highlightTone?: 'blue' | 'rose';
  subtitle?: string;
  className?: string;
};

export function SectionHeading({
  kicker,
  title,
  highlight,
  highlightTone = 'blue',
  subtitle,
  className,
}: SectionHeadingProps) {
  const [before, after] = highlight ? title.split(highlight) : [title, ''];

  return (
    <div className={cn('wn-section-head', className)}>
      <span className="wn-kicker">{kicker}</span>
      <h2 className="wn-section-title">
        {highlight ? (
          <>
            {before}
            <span className={cn('wn-highlight', highlightTone === 'rose' && 'wn-highlight--rose')}>
              {highlight}
            </span>
            {after}
          </>
        ) : (
          title
        )}
      </h2>
      {subtitle ? <p className="wn-section-subtitle">{subtitle}</p> : null}
    </div>
  );
}
