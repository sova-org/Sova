<script lang="ts">
  import { X, ExternalLink } from 'lucide-svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';

  interface Props {
    open: boolean;
  }

  let { open: isOpen = $bindable() }: Props = $props();

  function close() {
    isOpen = false;
  }

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      close();
    }
  }

  async function openSova() {
    await openUrl('https://sova.livecoding.fr');
  }

  async function openCookie() {
    await openUrl('https://cookie.paris');
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div class="modal-overlay" onclick={handleOverlayClick} role="presentation">
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="about-title">
      <button class="close-button" onclick={close} aria-label="Close">
        <X size={16} />
      </button>

      <div class="content">
        <img src="/logo.png" alt="Sova logo" class="logo" />

        <h1 id="about-title" class="title">Sova</h1>
        <span class="version">v0.1.0</span>

        <p class="description">
          Lorem ipsum dolor sit amet, consectetur adipiscing elit.
          Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
        </p>

        <div class="team">
          <span class="label">Team</span>
          <div class="names">
            <span>Raphaël Maurice Forment</span>
            <span>Loïg Jezequel</span>
            <span>Tanguy Dubois</span>
          </div>
        </div>

        <div class="links">
          <button class="link" onclick={openSova}>
            sova.livecoding.fr
            <ExternalLink size={11} />
          </button>
          <button class="link" onclick={openCookie}>
            cookie.paris
            <ExternalLink size={11} />
          </button>
          <button class="link" onclick={() => openUrl('https://www.athenor.com')}>
            athenor.com
            <ExternalLink size={11} />
          </button>
          <button class="link" onclick={() => openUrl('https://toplap.org')}>
            toplap.org
            <ExternalLink size={11} />
          </button>
        </div>

        <button class="license-link" onclick={() => openUrl('https://www.gnu.org/licenses/agpl-3.0.html')}>
          AGPL-3.0 License <ExternalLink size={10} />
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--colors-background, #1e1e1e);
    border: 1px solid var(--colors-border, #333);
    padding: 24px;
    min-width: 320px;
    max-width: 400px;
    position: relative;
  }

  .close-button {
    position: absolute;
    top: 12px;
    right: 12px;
    background: none;
    border: none;
    color: var(--colors-text-secondary, #888);
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-button:hover {
    color: var(--colors-text, #fff);
  }

  .content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 12px;
  }

  .logo {
    width: 80px;
    height: 80px;
    margin-bottom: 4px;
  }

  .title {
    font-family: monospace;
    font-size: 24px;
    font-weight: 700;
    color: var(--colors-text, #fff);
    margin: 0;
    letter-spacing: 1px;
  }

  .version {
    font-family: monospace;
    font-size: 12px;
    color: var(--colors-text-secondary, #888);
  }

  .description {
    font-family: monospace;
    font-size: 12px;
    line-height: 1.6;
    color: var(--colors-text, #fff);
    margin: 8px 0;
    max-width: 300px;
  }

  .team {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 8px 0;
  }

  .label {
    font-family: monospace;
    font-size: 11px;
    color: var(--colors-text-secondary, #666);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .names {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-family: monospace;
    font-size: 12px;
    color: var(--colors-text, #fff);
  }

  .links {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
    margin: 8px 0;
  }

  .link {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: monospace;
    font-size: 11px;
    color: var(--colors-text, #fff);
    background: none;
    border: 1px solid var(--colors-border, #333);
    cursor: pointer;
    padding: 8px 12px;
  }

  .link:hover {
    border-color: var(--colors-accent, #0e639c);
    color: var(--colors-accent, #0e639c);
  }

  .license-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-top: 8px;
    font-family: monospace;
    font-size: 11px;
    color: var(--colors-text-secondary, #888);
    background: none;
    border: none;
    cursor: pointer;
  }

  .license-link:hover {
    color: var(--colors-text, #fff);
    text-decoration: underline;
  }
</style>
