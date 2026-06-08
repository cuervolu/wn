import * as React from 'react';
import { cn } from '@/lib/cn';

type StickerButtonProps = React.ComponentPropsWithoutRef<'a'> & {
  kind?: 'rose' | 'ghost' | 'blue';
  size?: 'sm' | 'md' | 'lg';
};

export function StickerButton({
  className,
  kind = 'rose',
  size = 'md',
  ...props
}: StickerButtonProps) {
  return (
    <a
      className={cn(
        'wn-sticker',
        `wn-sticker--${kind}`,
        `wn-sticker--${size}`,
        className,
      )}
      {...props}
    />
  );
}
