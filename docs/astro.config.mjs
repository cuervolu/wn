// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import wnGrammar from './src/grammars/wn.tmLanguage.json';

export default defineConfig({
  integrations: [
    starlight({
      title: 'WN++',
      description: 'Un lenguaje de programación chileno, simple y directo.',
      favicon: './src/assets/favicon.svg',
      logo: {
        src: './src/assets/favicon.svg',
      },

     social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/cuervolu/wn' }],

      customCss: ['./src/styles/custom.css'],


      expressiveCode: {
        themes: ['one-dark-pro'],
 		shiki: {                 
          langs: [wnGrammar],
        },
      },

      sidebar: [
        {
          label: '¿Qué es WN++?',
          items: [
            { label: 'Introducción', link: '/' },
            { label: 'Instalación', link: '/instalacion' },
            { label: 'Filosofía', link: '/filosofia' },
            { label: 'Hoja de ruta', link: '/roadmap' },
          ],
        },
        {
          label: 'El lenguaje',
          items: [
            { label: 'Variables', link: '/lenguaje/variables' },
            { label: 'Funciones', link: '/lenguaje/funciones' },
            { label: 'Control de flujo', link: '/lenguaje/control-flujo' },
            { label: 'Tipos de datos', link: '/lenguaje/tipos' },
            { label: 'Errores', link: '/lenguaje/errores' },
          ],
        },
        // {
        //   label: 'Internals',
        //   collapsed: true,
        //   items: [
        //     { label: 'Flujo', link: '/internals/pipeline' },
        //     { label: 'Lexer', link: '/internals/lexer' },
        //     { label: 'Parser', link: '/internals/parser' },
        //     { label: 'Intérprete', link: '/internals/interprete' },
        //   ],
        // },
      ],
    }),
  ],

  site: 'https://cuervolu.github.io',
  base: '/wn',
});
