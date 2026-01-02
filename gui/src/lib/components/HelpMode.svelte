<script lang="ts">
    import { isHelpModeActive, exitHelpMode } from "$lib/stores/helpMode";
    import { helpContent, type HelpEntry } from "$lib/help/helpContent";

    let tooltipX = $state(0);
    let tooltipY = $state(0);
    let hoveredHelp = $state<HelpEntry | null>(null);
    // svelte-ignore non_reactive_update
    let tooltipEl: HTMLDivElement;

    $effect(() => {
        if ($isHelpModeActive) {
            document.body.classList.add("help-mode-active");
        } else {
            document.body.classList.remove("help-mode-active");
        }
    });

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape" && $isHelpModeActive) {
            exitHelpMode();
        }
    }

    function handleMouseMove(event: MouseEvent) {
        if (!$isHelpModeActive) return;

        const target = event.target as HTMLElement;
        const helpElement = target.closest(
            "[data-help-id]",
        ) as HTMLElement | null;

        if (helpElement) {
            const helpId = helpElement.getAttribute("data-help-id");
            if (helpId && helpContent[helpId]) {
                hoveredHelp = helpContent[helpId];
                positionTooltip(event.clientX, event.clientY);
                return;
            }
        }

        hoveredHelp = null;
    }

    function positionTooltip(mouseX: number, mouseY: number) {
        const offsetX = 16;
        const offsetY = 16;
        const padding = 12;

        let x = mouseX + offsetX;
        let y = mouseY + offsetY;

        // Get tooltip dimensions after render
        requestAnimationFrame(() => {
            if (!tooltipEl) return;

            const rect = tooltipEl.getBoundingClientRect();
            const viewportWidth = window.innerWidth;
            const viewportHeight = window.innerHeight;

            // Flip horizontally if overflowing right
            if (x + rect.width + padding > viewportWidth) {
                x = mouseX - rect.width - offsetX;
            }

            // Flip vertically if overflowing bottom
            if (y + rect.height + padding > viewportHeight) {
                y = mouseY - rect.height - offsetY;
            }

            // Ensure minimum bounds
            x = Math.max(padding, x);
            y = Math.max(padding, y);

            tooltipX = x;
            tooltipY = y;
        });

        // Initial position
        tooltipX = x;
        tooltipY = y;
    }
</script>

<svelte:window onkeydown={handleKeydown} onmousemove={handleMouseMove} />

{#if $isHelpModeActive}
    <div class="help-overlay"></div>

    {#if hoveredHelp}
        <div
            class="help-tooltip"
            bind:this={tooltipEl}
            style="left: {tooltipX}px; top: {tooltipY}px;"
        >
            <h4 class="tooltip-title">{hoveredHelp.title}</h4>
            <p class="tooltip-description">{hoveredHelp.description}</p>
        </div>
    {/if}
{/if}

<style>
    .help-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.125);
        z-index: 999;
        pointer-events: none;
    }

    .help-tooltip {
        position: fixed;
        z-index: 1001;
        background: var(--colors-background, #1e1e1e);
        border: 1px solid var(--colors-accent, #0e639c);
        padding: 12px 16px;
        max-width: 280px;
        pointer-events: none;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    }

    .tooltip-title {
        margin: 0 0 6px 0;
        font-family: monospace;
        font-size: 13px;
        font-weight: 600;
        color: var(--colors-text, #fff);
    }

    .tooltip-description {
        margin: 0;
        font-family: monospace;
        font-size: 11px;
        line-height: 1.5;
        color: var(--colors-text-secondary, #888);
    }
</style>
