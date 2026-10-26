<script lang="ts">
  import { editor } from '../stores/editor.svelte';

  const editorState = $derived(editor.state);
  const environment = $derived(editor.environment);
  /** The latest change was made by an AI agent over MCP. */
  const aiChange = $derived(editorState?.undo?.startsWith('AI: ') ? editorState.undo.slice(4) : null);
  const fileName = $derived(editorState?.file?.split(/[\\/]/).pop() ?? null);

  async function copyMcpUrl(): Promise<void> {
    if (!environment?.mcp_url) return;
    await navigator.clipboard.writeText(environment.mcp_url);
    editor.notify('MCP endpoint copied. Add it to your AI client as a streamable HTTP server.');
  }
</script>

<footer class="status">
  <div class="side">
    {#if editor.message}
      <span class="message" class:error={editor.message.error}>{editor.message.text}</span>
    {:else if aiChange}
      <span class="pill ai" title="Undo to revert it"><span class="dot"></span>AI · {aiChange}</span>
    {/if}
    {#if editorState?.problems.length}
      <span class="pill error" title={editorState.problems.join('\n')}>
        <span class="dot"></span>{editorState.problems.length} chip-profile problem{editorState.problems.length === 1 ? '' : 's'}
      </span>
    {/if}
  </div>

  <div class="side end">
    {#if fileName}<span class="file" title={editorState?.file}>{fileName}</span>{/if}
    {#if environment}
      {#if environment.mcp_url}
        <button class="pill ok" onclick={() => void copyMcpUrl()} title={`MCP endpoint ${environment.mcp_url} — click to copy`}>
          <span class="dot"></span>MCP
        </button>
      {:else}
        <span class="pill error" title={environment.mcp_error ?? ''}><span class="dot"></span>MCP off</span>
      {/if}
      <span
        class="pill"
        class:ok={environment.audio_device}
        class:error={!environment.audio_device}
        title={environment.audio_device ?? environment.audio_error ?? ''}
      >
        <span class="dot"></span>{environment.audio_device ? 'Audio' : 'No audio'}
      </span>
    {/if}
  </div>
</footer>

<style>
  .status {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    height: 28px;
    padding: 0 12px;
    background: var(--panel);
    border-top: 1px solid var(--border);
    font-size: 11px;
    white-space: nowrap;
  }

  .side {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    overflow: hidden;
  }

  .end {
    flex: none;
  }

  .message {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .message.error {
    color: var(--danger);
  }

  .file {
    color: var(--muted);
  }

  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 1px 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--bg);
    color: var(--muted);
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  button.pill:hover {
    border-color: var(--accent);
    color: var(--text);
  }

  .dot {
    width: 6px;
    height: 6px;
    flex: none;
    border-radius: 50%;
    background: var(--muted);
  }

  .ok .dot {
    background: var(--accent);
  }

  .error {
    color: var(--danger);
  }

  .error .dot {
    background: var(--danger);
  }

  .ai {
    color: var(--accent-2);
  }

  .ai .dot {
    background: var(--accent-2);
  }
</style>
