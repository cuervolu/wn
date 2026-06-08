import { cn } from '@/lib/cn';
import { logoSrc } from '@/components/landing/content';

export function DocsBrand({ className }: { className?: string }) {
  return (
    <a className={cn('wn-docs-brand', className)} href="/" aria-label="Volver al landing de WN++">
      <img className="wn-docs-brand__logo" src={logoSrc} alt="WN++" />
      <span className="wn-docs-brand__badge">Docs</span>
    </a>
  );
}
