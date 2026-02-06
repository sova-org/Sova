<script lang="ts">
    import { currentView, viewState } from "$lib/stores/viewState";
    import Login from "./Login.svelte";
    import SceneView from "./SceneView.svelte";
    import DevicesView from "./DevicesView.svelte";
    import LogView from "./LogView.svelte";
    import ChatView from "./ChatView.svelte";
    import ProjectsView from "./ProjectsView.svelte";
    import SettingsPanel from "./SettingsPanel.svelte";
    import ViewNavigator from "./ViewNavigator.svelte";

    function handleLoginSuccess() {
        viewState.navigateTo("SCENE");
    }
</script>

<div class="view-container">
    {#if $currentView === "LOGIN"}
        <Login onConnected={handleLoginSuccess} />
    {:else if $currentView === "SCENE"}
        <SceneView />
    {:else if $currentView === "DEVICES"}
        <DevicesView />
    {:else if $currentView === "LOGS"}
        <LogView />
    {:else if $currentView === "CHAT"}
        <ChatView />
    {:else if $currentView === "PROJECTS"}
        <ProjectsView />
    {:else if $currentView === "CONFIG"}
        <SettingsPanel />
    {/if}
</div>

<ViewNavigator />

<style>
    .view-container {
        width: 100%;
        height: 100%;
        overflow: hidden;
    }
</style>
