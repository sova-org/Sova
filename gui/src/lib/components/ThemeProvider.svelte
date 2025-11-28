<script lang="ts">
  import { currentTheme, currentTransparency, currentZoom, config } from '$lib/stores/config';
  import { hexToRgba } from '$lib/utils/colorUtils';
  import type { Snippet } from 'svelte';

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();

  function toKebabCase(str: string): string {
    return str.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase();
  }

  function flattenTheme(theme: any, transparency: number): Record<string, string> {
    const result: Record<string, string> = {};
    const alpha = transparency / 100;

    const backgroundKeys = ['background', 'surface', 'gutter', 'activeLineGutter', 'activeLine', 'selection'];

    for (const [section, values] of Object.entries(theme)) {
      if (section === 'name' || typeof values !== 'object') continue;

      for (const [key, value] of Object.entries(values as Record<string, string>)) {
        const cssKey = `--${section}-${toKebabCase(key)}`;
        if (typeof value === 'string' && value.startsWith('#') && backgroundKeys.includes(key)) {
          result[cssKey] = hexToRgba(value, alpha);
        } else {
          result[cssKey] = value;
        }
      }
    }

    return result;
  }

  const themeVars = $derived(flattenTheme($currentTheme, $currentTransparency));

  const appearanceFont = $derived($config?.appearance?.font_family || 'monospace');

  const styleString = $derived(
    Object.entries(themeVars)
      .map(([key, value]) => `${key}: ${value}`)
      .concat(`--appearance-font-family: ${appearanceFont}`)
      .concat(`zoom: ${$currentZoom}`)
      .join('; ')
  );
</script>

<div class="theme-provider" style={styleString}>
  {@render children()}
</div>

<style>
  .theme-provider {
    display: contents;
  }
</style>
