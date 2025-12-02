<script lang="ts">
    import { Pane, Splitpanes } from "svelte-splitpanes";
    import { paneLayout, type PaneNode } from "$lib/stores/paneState";
    import PaneContainer from "./PaneContainer.svelte";
    import PaneNodeRenderer from "./PaneNodeRenderer.svelte";

    interface Props {
        node: PaneNode;
        isOnlyPane: boolean;
    }

    let { node, isOnlyPane }: Props = $props();

    function handleResize(
        splitId: string,
        event: CustomEvent<{ size: number }[]>,
    ) {
        const detail = event.detail;
        if (detail.length === 2) {
            paneLayout.updateSizes(splitId, [detail[0].size, detail[1].size]);
        }
    }
</script>

{#if node.type === "leaf"}
    <PaneContainer paneId={node.id} viewType={node.viewType} {isOnlyPane} />
{:else}
    <Splitpanes
        horizontal={node.direction === "horizontal"}
        theme=""
        on:resized={(e) => handleResize(node.id, e)}
    >
        <Pane minSize={10} size={node.sizes[0]}>
            <PaneNodeRenderer node={node.children[0]} isOnlyPane={false} />
        </Pane>
        <Pane minSize={10} size={node.sizes[1]}>
            <PaneNodeRenderer node={node.children[1]} isOnlyPane={false} />
        </Pane>
    </Splitpanes>
{/if}
