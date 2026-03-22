import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import ts from 'typescript-eslint';

export default ts.config(
	js.configs.recommended,
	...ts.configs.recommended,
	...svelte.configs['flat/recommended'],
	{
		rules: {
			// Svelte 5 特定规则
			'svelte/no-unused-svelte-ignore': 'error',
		},
	},
	{
		ignores: ['.svelte-kit/**', 'build/**', 'node_modules/**'],
	}
);
