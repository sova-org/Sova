<script lang="ts">
    import {
        currentThemeTransformed,
        currentZoom,
        config,
    } from "$lib/stores/config";
    import type { Snippet } from "svelte";

    interface Props {
        children: Snippet;
    }

    let { children }: Props = $props();

    function toKebabCase(str: string): string {
        return str.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase();
    }

    function flattenTheme(theme: any): Record<string, string> {
        const result: Record<string, string> = {};

        for (const [section, values] of Object.entries(theme)) {
            if (section === "name" || typeof values !== "object") continue;

            for (const [key, value] of Object.entries(
                values as Record<string, string>,
            )) {
                const cssKey = `--${section}-${toKebabCase(key)}`;
                result[cssKey] = value;
            }
        }

        return result;
    }

    const themeVars = $derived(flattenTheme($currentThemeTransformed));

    const appearanceFont = $derived(
        $config?.appearance?.font_family || "monospace",
    );

    const styleString = $derived(
        Object.entries(themeVars)
            .map(([key, value]) => `${key}: ${value}`)
            .concat(`--appearance-font-family: ${appearanceFont}`)
            .concat(`zoom: ${$currentZoom}`)
            .join("; "),
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
