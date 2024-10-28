import { join } from 'path';
import type { Config } from 'tailwindcss';
import { skeleton } from '@skeletonlabs/tw-plugin';
import typography from '@tailwindcss/typography';
import forms from '@tailwindcss/forms';

const config = {
  content: [
		'./src/**/*.{html,js,svelte,ts}',
		join(require.resolve(
			'@skeletonlabs/skeleton'),
			'../**/*.{html,js,svelte,ts}'
		)
  ],

  theme: {
    extend: {
      // Add your customizations here if needed
    }
  },

  plugins: [
    forms,
    typography,
    skeleton({
      // Optional theme preset or custom settings (light, modern, etc.)
      themes: {
        preset: [
          {
            name: 'skeleton',
            enhancements: true,
          },
          {
            name: 'wintry',
            enhancements: true,
          },
          {
            name: 'modern',
            enhancements: true,
          },
          {
            name: 'rocket',
            enhancements: true,
          },
          {
            name: 'seafoam',
            enhancements: true,
          },
          {
            name: 'vintage',
            enhancements: true,
          },
          {
            name: 'sahara',
            enhancements: true,
          },
          {
            name: 'hamlindigo',
            enhancements: true,
          },
          {
            name: 'gold-nouveau',
            enhancements: true,
          },
          {
            name: 'crimson',
            enhancements: true,
          }
        ],
      },
    })
  ]
} satisfies Config;

export default config;
						

