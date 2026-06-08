import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import { DocsBrand } from '@/components/docs/brand';
import { gitConfig } from './shared';

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: DocsBrand,
      url: '/',
    },
    githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
  };
}
