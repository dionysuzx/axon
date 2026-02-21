// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Axon',
			description: 'Validate and refactor prompt filenames',
			sidebar: [
				{ label: 'Introduction', slug: 'index' },
				{ label: 'Getting Started', slug: 'getting-started' },
				{ label: 'Naming Convention', slug: 'naming-convention' },
				{
					label: 'Commands',
					items: [
						{ label: 'axon health', slug: 'commands/health' },
						{ label: 'axon validate', slug: 'commands/validate' },
						{ label: 'axon parse', slug: 'commands/parse' },
						{ label: 'axon refactor', slug: 'commands/refactor' },
						{ label: 'axon stats', slug: 'commands/stats' },
						{ label: 'axon d', slug: 'commands/daily' },
						{ label: 'axon (TUI)', slug: 'commands/tui' },
					],
				},
				{
					label: 'Guides',
					items: [
						{ label: 'Refactoring Workflows', slug: 'guides/refactoring' },
						{ label: 'Notes', slug: 'guides/daily-notes' },
						{ label: 'CI Integration', slug: 'guides/ci-integration' },
					],
				},
				{
					label: 'Reference',
					items: [
						{ label: 'Configuration', slug: 'reference/config' },
						{ label: 'Exit Codes', slug: 'reference/exit-codes' },
						{ label: 'Pattern Syntax', slug: 'reference/patterns' },
					],
				},
			],
		}),
	],
});
