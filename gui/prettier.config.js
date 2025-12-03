export default {
	useTabs: true,
	singleQuote: true,
	trailingComma: 'es5',
	plugins: ['prettier-plugin-svelte'],
	overrides: [{ files: '*.svelte', options: { parser: 'svelte' } }],
};
