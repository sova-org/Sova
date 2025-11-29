<script lang="ts">
	import { chatMessages, type ChatMessage } from '$lib/stores/collaboration';
	import { sendChat } from '$lib/api/client';
	import { clientConfig } from '$lib/stores/config';
	import { Send } from 'lucide-svelte';

	let messageInput = $state('');
	let messagesContainer: HTMLDivElement;
	let autoScroll = $state(true);

	function getUserColor(username: string): string {
		let hash = 0;
		for (let i = 0; i < username.length; i++) {
			hash = username.charCodeAt(i) + ((hash << 5) - hash);
		}
		const hue = Math.abs(hash) % 360;
		return `hsl(${hue}, 70%, 65%)`;
	}

	function formatTimestamp(timestamp: number): string {
		const date = new Date(timestamp);
		const hours = String(date.getHours()).padStart(2, '0');
		const minutes = String(date.getMinutes()).padStart(2, '0');
		return `${hours}:${minutes}`;
	}

	async function handleSendMessage() {
		const trimmed = messageInput.trim();
		if (trimmed && $clientConfig?.nickname) {
			chatMessages.update((msgs) => [
				...msgs,
				{
					user: $clientConfig.nickname,
					message: trimmed,
					timestamp: Date.now()
				}
			]);
			messageInput = '';
			sendChat(trimmed);
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			handleSendMessage();
		}
	}

	function scrollToBottom() {
		if (autoScroll && messagesContainer) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight;
		}
	}

	$effect(() => {
		$chatMessages;
		requestAnimationFrame(scrollToBottom);
	});
</script>

<div class="chat-view">
	<div class="toolbar">
		<h2 class="title">CHAT</h2>
		<label class="auto-scroll-toggle">
			<input type="checkbox" bind:checked={autoScroll} />
			Auto-scroll
		</label>
	</div>

	<div class="messages-container" bind:this={messagesContainer}>
		{#if $chatMessages.length === 0}
			<div class="empty-state">No messages yet</div>
		{:else}
			{#each $chatMessages as msg (msg.timestamp)}
				<div class="message">
					<span class="timestamp">{formatTimestamp(msg.timestamp)}</span>
					<span class="username" style="color: {getUserColor(msg.user)}">{msg.user}</span>
					<span class="text">{msg.message}</span>
				</div>
			{/each}
		{/if}
	</div>

	<div class="input-area">
		<input
			type="text"
			bind:value={messageInput}
			onkeydown={handleKeydown}
			placeholder="Type a message..."
			class="message-input"
		/>
		<button class="send-button" onclick={handleSendMessage} title="Send message">
			<Send size={16} />
		</button>
	</div>
</div>

<style>
	.chat-view {
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--colors-background);
	}

	.toolbar {
		height: 40px;
		background-color: var(--colors-surface, #252525);
		border-bottom: 1px solid var(--colors-border, #333);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 16px;
	}

	.title {
		margin: 0;
		font-size: 13px;
		font-weight: 600;
		color: var(--colors-text, #fff);
		font-family: monospace;
	}

	.auto-scroll-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		color: var(--colors-text-secondary, #888);
		font-family: monospace;
		cursor: pointer;
	}

	.auto-scroll-toggle input[type='checkbox'] {
		appearance: none;
		-webkit-appearance: none;
		width: 12px;
		height: 12px;
		border: 1px solid var(--colors-border, #333);
		background-color: var(--colors-background, #1e1e1e);
		cursor: pointer;
		position: relative;
		flex-shrink: 0;
	}

	.auto-scroll-toggle input[type='checkbox']:checked {
		background-color: var(--colors-accent, #0e639c);
		border-color: var(--colors-accent, #0e639c);
	}

	.auto-scroll-toggle input[type='checkbox']:checked::after {
		content: '';
		position: absolute;
		left: 3px;
		top: 0px;
		width: 4px;
		height: 8px;
		border: solid var(--colors-text, #fff);
		border-width: 0 2px 2px 0;
		transform: rotate(45deg);
	}

	.auto-scroll-toggle input[type='checkbox']:hover {
		border-color: var(--colors-accent, #0e639c);
	}

	.messages-container {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 8px 0;
	}

	.empty-state {
		color: var(--colors-text-secondary, #888);
		font-family: monospace;
		font-size: 13px;
		text-align: center;
		padding: 32px;
	}

	.message {
		display: flex;
		gap: 8px;
		font-family: monospace;
		font-size: 13px;
		padding: 4px 16px;
		box-sizing: border-box;
	}

	.message:hover {
		background-color: var(--colors-surface, #2d2d2d);
	}

	.timestamp {
		color: var(--colors-text-secondary, #888);
		flex-shrink: 0;
		font-size: 11px;
	}

	.username {
		font-weight: 600;
		flex-shrink: 0;
	}

	.text {
		color: var(--colors-text, #fff);
		flex: 1;
		word-break: break-word;
	}

	.input-area {
		display: flex;
		gap: 8px;
		padding: 12px 16px;
		border-top: 1px solid var(--colors-border, #333);
		background-color: var(--colors-surface, #252525);
	}

	.message-input {
		flex: 1;
		background-color: var(--colors-background, #1e1e1e);
		border: 1px solid var(--colors-border, #333);
		color: var(--colors-text, #fff);
		padding: 8px 12px;
		font-family: monospace;
		font-size: 13px;
	}

	.message-input::placeholder {
		color: var(--colors-text-secondary, #888);
	}

	.message-input:focus {
		outline: none;
		border-color: var(--colors-accent, #0e639c);
	}

	.send-button {
		background: none;
		border: 1px solid var(--colors-border, #333);
		color: var(--colors-text-secondary, #888);
		padding: 8px 12px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s;
	}

	.send-button:hover {
		border-color: var(--colors-accent, #0e639c);
		color: var(--colors-accent, #0e639c);
	}
</style>
