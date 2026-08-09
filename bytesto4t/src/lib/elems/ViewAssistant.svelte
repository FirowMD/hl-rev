<script lang="ts">
  import { Channel, invoke } from "@tauri-apps/api/core";
  import {
    Bot,
    Brain,
    Check,
    ChevronDown,
    Copy,
    LoaderCircle,
    LogIn,
    LogOut,
    MessageSquare,
    MessageSquarePlus,
    PanelLeftClose,
    PanelLeftOpen,
    Plus,
    RefreshCw,
    Save,
    Send,
    Settings,
    ShieldCheck,
    Square,
    Trash2,
    Wrench
  } from "@lucide/svelte";
  import createDOMPurify, { type DOMPurify } from "dompurify";
  import { marked } from "marked";
  import { onMount, tick } from "svelte";

  type AssistantSettings = {
    model: string;
    reasoning_effort: string;
    bypass_proxy: boolean;
    disable_tls: boolean;
    proxy_url: string;
    privacy_disclosure_version: number;
  };

  type ToolActivity = {
    name: string;
    arguments: Record<string, unknown>;
    success: boolean;
    summary: string;
  };

  type Usage = {
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
  };

  type ChatMessage = {
    role: "user" | "assistant";
    content: string;
    model?: string;
    reasoning?: string[];
    toolActivity?: ToolActivity[];
    usage?: Usage;
  };

  type Chat = {
    id: string;
    title: string;
    createdAt: number;
    updatedAt: number;
    messages: ChatMessage[];
  };

  type ChatStore = {
    activeChatId: string;
    chats: Chat[];
  };

  type AssistantStatus = {
    authenticated: boolean;
    settings: AssistantSettings;
    models: string[];
    privacyDisclosureAccepted: boolean;
    privacyDisclosureVersion: number;
  };

  type AssistantReply = {
    content: string;
    model: string;
    reasoning: string[];
    toolActivity: ToolActivity[];
    usage: Usage;
  };

  type AssistantStreamEvent =
    | { type: "roundStarted"; round: number; finalizing: boolean }
    | { type: "reasoningDelta"; delta: string }
    | { type: "outputDelta"; delta: string }
    | { type: "toolStarted"; name: string; arguments: Record<string, unknown> }
    | { type: "toolFinished"; activity: ToolActivity };

  const STORAGE_KEY = "bytesto4t.assistant.chats.v2";
  const LEGACY_STORAGE_KEY = "bytesto4t.assistant.chat.v1";
  const ALTERNATE_LEGACY_STORAGE_KEY = "bytesto4t.assistant.chats.v1";
  const NEW_CHAT_TITLE = "New chat";
  const defaultSettings: AssistantSettings = {
    model: "",
    reasoning_effort: "medium",
    bypass_proxy: false,
    disable_tls: false,
    proxy_url: "",
    privacy_disclosure_version: 0
  };

  let authenticated = $state(false);
  let settings = $state<AssistantSettings>({ ...defaultSettings });
  let models = $state<string[]>([]);
  let chats = $state<Chat[]>([]);
  let activeChatId = $state("");
  let messages = $derived(chats.find((chat) => chat.id === activeChatId)?.messages ?? []);
  let orderedChats = $derived([...chats].sort((a, b) => b.updatedAt - a.updatedAt));
  let prompt = $state("");
  let busy = $state(false);
  let authenticating = $state(false);
  let loadingModels = $state(false);
  let settingsOpen = $state(false);
  let errorMessage = $state("");
  let settingsSaved = $state(false);
  let sidebarOpen = $state(true);
  let liveReasoning = $state<string[]>([]);
  let liveReasoningRound = $state(0);
  let liveOutput = $state("");
  let liveToolActivity = $state<ToolActivity[]>([]);
  let liveToolName = $state("");
  let liveRound = $state(0);
  let liveFinalizing = $state(false);
  let copiedMessageKey = $state("");
  let privacyDisclosureOpen = $state(false);
  let privacyDisclosureAccepted = $state(false);
  let privacyDisclosureVersion = $state(0);
  let acceptingPrivacy = $state(false);
  let lastSavedDisableTls = $state(false);
  let tlsBypassConfirmed = $state(false);
  let historyBusy = $state(false);
  let tauriAvailable = false;
  let encryptedPersistenceAvailable = $state(false);
  let persistenceTail: Promise<void> = Promise.resolve();
  let requestSequence = 0;
  let messagesContainer = $state<HTMLDivElement>();
  let markdownSanitizer: DOMPurify | undefined;
  let copyResetTimer: ReturnType<typeof window.setTimeout> | undefined;
  const markdownCache = new Map<string, string>();

  onMount(() => {
    markdownSanitizer = createDOMPurify(window);
    settingsOpen = new URLSearchParams(window.location.search).get("assistantSettings") === "1";
    tauriAvailable = "__TAURI_INTERNALS__" in window;
    void initializeAssistant();
  });

  async function initializeAssistant() {
    if (tauriAvailable) {
      await restoreChats();
      await loadStatus();
    } else {
      ensureActiveChat();
    }
  }

  function newChatRecord(): Chat {
    const now = Date.now();
    const id = globalThis.crypto?.randomUUID?.() ?? `chat-${now}-${Math.random().toString(16).slice(2)}`;
    return { id, title: NEW_CHAT_TITLE, createdAt: now, updatedAt: now, messages: [] };
  }

  function titleFor(messages: ChatMessage[]) {
    const content = messages.find((message) => message.role === "user")?.content.trim() ?? "";
    const singleLine = content.replace(/\s+/g, " ");
    return singleLine.length > 42 ? `${singleLine.slice(0, 39)}...` : singleLine || NEW_CHAT_TITLE;
  }

  function ensureActiveChat() {
    if (chats.length === 0) {
      const chat = newChatRecord();
      chats = [chat];
      activeChatId = chat.id;
    }
  }

  function applyChatStore(store: ChatStore | null) {
    if (store && Array.isArray(store.chats)) {
      chats = store.chats;
      activeChatId = chats.some((chat) => chat.id === store.activeChatId)
        ? store.activeChatId
        : (chats[0]?.id ?? "");
    }
    ensureActiveChat();
  }

  function removePlaintextChatKeys() {
    localStorage.removeItem(STORAGE_KEY);
    localStorage.removeItem(LEGACY_STORAGE_KEY);
    localStorage.removeItem(ALTERNATE_LEGACY_STORAGE_KEY);
  }

  async function restoreChats() {
    try {
      const v2Json = localStorage.getItem(STORAGE_KEY);
      const v1Json =
        localStorage.getItem(LEGACY_STORAGE_KEY) ??
        localStorage.getItem(ALTERNATE_LEGACY_STORAGE_KEY);
      let store: ChatStore | null;
      if (v2Json !== null || v1Json !== null) {
        store = await invoke<ChatStore>("migrate_assistant_chats", { v2Json, v1Json });
        removePlaintextChatKeys();
      } else {
        store = await invoke<ChatStore | null>("load_assistant_chats");
      }
      encryptedPersistenceAvailable = true;
      applyChatStore(store);
      if (!store || store.chats.length === 0) {
        const persisted = await persistChats();
        if (!persisted) encryptedPersistenceAvailable = false;
      }
    } catch {
      encryptedPersistenceAvailable = false;
      applyChatStore(null);
      errorMessage =
        "Encrypted chat history or migration is unavailable. Existing plaintext migration data, if any, was not deleted; this session will not write Assistant chats to disk.";
    }
  }

  async function persistStore(store: ChatStore) {
    if (!tauriAvailable || !encryptedPersistenceAvailable) return true;
    const operation = persistenceTail.then(() =>
      invoke<void>("save_assistant_chats", { store })
    );
    persistenceTail = operation.catch(() => undefined);
    try {
      await operation;
      return true;
    } catch {
      errorMessage =
        "Could not update encrypted chat history. The latest changes remain session-only.";
      return false;
    }
  }

  function persistChats() {
    return persistStore({ activeChatId, chats });
  }

  async function updateActiveMessages(nextMessages: ChatMessage[]) {
    const now = Date.now();
    chats = chats.map((chat) =>
      chat.id === activeChatId
        ? {
            ...chat,
            title: chat.title === NEW_CHAT_TITLE ? titleFor(nextMessages) : chat.title,
            updatedAt: now,
            messages: nextMessages
          }
        : chat
    );
    await persistChats();
  }

  async function scrollToLatest() {
    await tick();
    messagesContainer?.scrollTo({ top: messagesContainer.scrollHeight, behavior: "smooth" });
  }

  async function loadStatus() {
    errorMessage = "";
    try {
      const status = await invoke<AssistantStatus>("get_assistant_status");
      authenticated = status.authenticated;
      settings = { ...defaultSettings, ...status.settings };
      lastSavedDisableTls = settings.disable_tls;
      models = status.models;
      privacyDisclosureAccepted = status.privacyDisclosureAccepted;
      privacyDisclosureVersion = status.privacyDisclosureVersion;
      privacyDisclosureOpen = !privacyDisclosureAccepted;
      if (authenticated && privacyDisclosureAccepted) await refreshModels(false);
    } catch {
      errorMessage = "Could not load Assistant status.";
    }
  }

  async function saveSettings(showConfirmation = true) {
    errorMessage = "";
    settingsSaved = false;
    try {
      await invoke("update_assistant_settings", {
        settings,
        confirmTlsBypass:
          settings.disable_tls && !lastSavedDisableTls && tlsBypassConfirmed
      });
      lastSavedDisableTls = settings.disable_tls;
      tlsBypassConfirmed = false;
      if (showConfirmation) {
        settingsSaved = true;
        window.setTimeout(() => (settingsSaved = false), 1600);
      }
    } catch (error) {
      errorMessage = String(error);
      throw error;
    }
  }

  async function connectChatGPT() {
    if (!privacyDisclosureAccepted) {
      privacyDisclosureOpen = true;
      return;
    }
    authenticating = true;
    errorMessage = "";
    try {
      await saveSettings(false);
      const status = await invoke<AssistantStatus>("start_chatgpt_oauth");
      authenticated = status.authenticated;
      models = status.models;
      if (!settings.model && models.length > 0) {
        settings = { ...settings, model: models[0] };
        await saveSettings(false);
      }
    } catch (error) {
      errorMessage = String(error);
    } finally {
      authenticating = false;
    }
  }

  async function disconnectChatGPT() {
    errorMessage = "";
    try {
      await invoke("disconnect_chatgpt");
      authenticated = false;
      models = [];
    } catch (error) {
      errorMessage = String(error);
    }
  }

  async function refreshModels(reportError = true) {
    loadingModels = true;
    if (reportError) errorMessage = "";
    try {
      models = await invoke<string[]>("discover_chatgpt_models");
      if (!settings.model && models.length > 0) {
        settings = { ...settings, model: models[0] };
        await saveSettings(false);
      }
    } catch (error) {
      if (reportError) errorMessage = String(error);
    } finally {
      loadingModels = false;
    }
  }

  function createChat() {
    if (busy || historyBusy) return;
    const chat = newChatRecord();
    chats = [chat, ...chats];
    activeChatId = chat.id;
    prompt = "";
    errorMessage = "";
    void persistChats();
  }

  function selectChat(id: string) {
    if (busy || historyBusy || id === activeChatId) return;
    activeChatId = id;
    errorMessage = "";
    void persistChats();
    void scrollToLatest();
  }

  async function removeChat(event: MouseEvent, id: string) {
    event.stopPropagation();
    if (busy || historyBusy) return;
    const chat = chats.find((item) => item.id === id);
    if (!chat || !window.confirm(`Remove "${chat.title}"?`)) return;
    const remaining = chats.filter((item) => item.id !== id);
    const nextActiveChatId = activeChatId === id ? (remaining[0]?.id ?? "") : activeChatId;
    historyBusy = true;
    try {
      const removed = await persistStore({ activeChatId: nextActiveChatId, chats: remaining });
      if (!removed) return;
      chats = remaining;
      activeChatId = nextActiveChatId;
    } finally {
      historyBusy = false;
    }
  }

  async function removeAllChats() {
    if (
      busy ||
      historyBusy ||
      chats.length === 0 ||
      !window.confirm("Remove all assistant chats?")
    )
      return;
    historyBusy = true;
    try {
      if (tauriAvailable) {
        const operation = persistenceTail.then(() => invoke<void>("delete_assistant_chats"));
        persistenceTail = operation.catch(() => undefined);
        await operation;
      }
      removePlaintextChatKeys();
      chats = [];
      activeChatId = "";
      prompt = "";
      errorMessage = "";
    } catch {
      errorMessage = "Could not remove encrypted Assistant chat history.";
    } finally {
      historyBusy = false;
    }
  }

  function resetLiveState() {
    liveReasoning = [];
    liveReasoningRound = 0;
    liveOutput = "";
    liveToolActivity = [];
    liveToolName = "";
    liveRound = 0;
    liveFinalizing = false;
  }

  function handleStreamEvent(event: AssistantStreamEvent, sequence: number) {
    if (sequence !== requestSequence) return;
    if (event.type === "roundStarted") {
      liveRound = event.round;
      liveFinalizing = event.finalizing;
      liveOutput = "";
      liveToolName = "";
    } else if (event.type === "reasoningDelta") {
      if (liveReasoningRound !== liveRound) {
        liveReasoning = [...liveReasoning, ""];
        liveReasoningRound = liveRound;
      }
      const index = liveReasoning.length - 1;
      liveReasoning = liveReasoning.map((part, partIndex) =>
        partIndex === index ? part + event.delta : part
      );
    } else if (event.type === "outputDelta") {
      liveOutput += event.delta;
    } else if (event.type === "toolStarted") {
      liveToolName = event.name;
    } else if (event.type === "toolFinished") {
      liveToolName = "";
      liveToolActivity = [...liveToolActivity, event.activity];
    }
    queueLatestScroll();
  }

  function usePrompt(value: string) {
    prompt = value;
    void submitMessage();
  }

  async function submitMessage() {
    const content = prompt.trim();
    if (!content || busy || !authenticated || !activeChatId) return;
    if (!privacyDisclosureAccepted) {
      privacyDisclosureOpen = true;
      return;
    }

    prompt = "";
    errorMessage = "";
    const userMessage: ChatMessage = { role: "user", content };
    const requestMessages = [...messages, userMessage];
    await updateActiveMessages(requestMessages);
    await scrollToLatest();

    busy = true;
    resetLiveState();
    const sequence = ++requestSequence;
    const onEvent = new Channel<AssistantStreamEvent>();
    onEvent.onmessage = (event) => handleStreamEvent(event, sequence);
    try {
      const reply = await invoke<AssistantReply>("send_assistant_message", {
        request: {
          messages: requestMessages.map(({ role, content }) => ({ role, content }))
        },
        onEvent
      });
      if (sequence !== requestSequence) return;
      await updateActiveMessages([
        ...requestMessages,
        {
          role: "assistant",
          content: reply.content,
          model: reply.model,
          reasoning: reply.reasoning,
          toolActivity: reply.toolActivity,
          usage: reply.usage
        }
      ]);
      await scrollToLatest();
    } catch (error) {
      if (sequence === requestSequence && !String(error).toLowerCase().includes("cancelled")) {
        errorMessage = String(error);
      }
    } finally {
      if (sequence === requestSequence) {
        busy = false;
        resetLiveState();
      }
    }
  }

  function handleTlsBypassChange(event: Event) {
    const enabled = (event.currentTarget as HTMLInputElement).checked;
    if (enabled && !settings.disable_tls) {
      const confirmed = window.confirm(
        "Disable TLS verification? This permits interception of ChatGPT credentials and analyzed content. Enable only for a network you fully trust."
      );
      if (!confirmed) {
        (event.currentTarget as HTMLInputElement).checked = false;
        return;
      }
      tlsBypassConfirmed = true;
    } else if (!enabled) {
      tlsBypassConfirmed = false;
    }
    settings = { ...settings, disable_tls: enabled };
  }

  async function acceptPrivacyDisclosure() {
    acceptingPrivacy = true;
    errorMessage = "";
    try {
      await invoke("accept_assistant_privacy_disclosure", {
        version: privacyDisclosureVersion
      });
      privacyDisclosureAccepted = true;
      privacyDisclosureOpen = false;
      settings = {
        ...settings,
        privacy_disclosure_version: privacyDisclosureVersion
      };
      if (authenticated) await refreshModels(false);
    } catch {
      errorMessage = "Could not save Assistant privacy consent.";
    } finally {
      acceptingPrivacy = false;
    }
  }

  async function cancelRequest() {
    requestSequence += 1;
    busy = false;
    resetLiveState();
    await invoke("cancel_assistant_message");
  }

  function handlePromptKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void submitMessage();
    }
  }

  function displayToolName(name: string) {
    return name.replaceAll("_", " ");
  }

  function renderMarkdown(content: string) {
    const cached = markdownCache.get(content);
    if (cached !== undefined) return cached;
    if (!markdownSanitizer) return "";

    const html = marked.parse(content, { async: false, breaks: true, gfm: true });
    const sanitized = markdownSanitizer.sanitize(html, {
      FORBID_ATTR: ["class", "id", "style"],
      FORBID_TAGS: ["img", "style"],
      USE_PROFILES: { html: true }
    });
    markdownCache.set(content, sanitized);
    return sanitized;
  }

  function copyWithFallback(content: string) {
    const textarea = document.createElement("textarea");
    textarea.value = content;
    textarea.setAttribute("readonly", "");
    textarea.style.position = "fixed";
    textarea.style.opacity = "0";
    document.body.append(textarea);
    textarea.select();
    const copied = document.execCommand("copy");
    textarea.remove();
    return copied;
  }

  async function copyAnswer(content: string, messageKey: string) {
    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(content);
      } else if (!copyWithFallback(content)) {
        throw new Error("Clipboard access is unavailable");
      }
      copiedMessageKey = messageKey;
      if (copyResetTimer) window.clearTimeout(copyResetTimer);
      copyResetTimer = window.setTimeout(() => (copiedMessageKey = ""), 1600);
    } catch (clipboardError) {
      try {
        if (!copyWithFallback(content)) throw clipboardError;
        copiedMessageKey = messageKey;
        if (copyResetTimer) window.clearTimeout(copyResetTimer);
        copyResetTimer = window.setTimeout(() => (copiedMessageKey = ""), 1600);
      } catch {
        errorMessage = "Could not copy the assistant answer to the clipboard.";
      }
    }
  }

  function formatChatTime(timestamp: number) {
    return new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" }).format(timestamp);
  }

  let scrollQueued = false;
  function queueLatestScroll() {
    if (scrollQueued) return;
    scrollQueued = true;
    requestAnimationFrame(() => {
      scrollQueued = false;
      messagesContainer?.scrollTo({ top: messagesContainer.scrollHeight });
    });
  }
</script>

<section class="assistant-view flex h-full min-h-0 flex-col overflow-hidden">
  <header class="assistant-toolbar flex h-10 shrink-0 items-center gap-2 border-b border-surface-700 px-3">
    <button
      type="button"
      class="icon-button"
      onclick={() => (sidebarOpen = !sidebarOpen)}
      aria-label={sidebarOpen ? "Hide chats" : "Show chats"}
      title={sidebarOpen ? "Hide chats" : "Show chats"}
      aria-expanded={sidebarOpen}
    >
      {#if sidebarOpen}<PanelLeftClose size={16} />{:else}<PanelLeftOpen size={16} />{/if}
    </button>
    <Bot size={17} strokeWidth={1.8} aria-hidden="true" />
    <h1 class="text-sm font-semibold text-surface-100">Assistant</h1>
    <span class:connected={authenticated} class="connection-dot ml-1" aria-hidden="true"></span>
    <span class="model-label min-w-0 truncate text-xs text-surface-400">
      {authenticated ? (settings.model || "ChatGPT") : "Disconnected"}
    </span>
    <div class="ml-auto flex items-center gap-1">
      <button
        type="button"
        class="icon-button"
        onclick={createChat}
        disabled={busy || historyBusy}
        aria-label="New chat"
        title="New chat"
      >
        <MessageSquarePlus size={16} />
      </button>
      {#if authenticated}
        <button
          type="button"
          class="icon-button"
          onclick={disconnectChatGPT}
          aria-label="Disconnect ChatGPT"
          title="Disconnect ChatGPT"
        >
          <LogOut size={16} />
        </button>
      {:else}
        <button
          type="button"
          class="connect-button"
          onclick={connectChatGPT}
          disabled={authenticating}
        >
          <LogIn size={15} />
          <span class="connect-label">{authenticating ? "Waiting for browser" : "Connect ChatGPT"}</span>
        </button>
      {/if}
      <button
        type="button"
        class="icon-button"
        onclick={() => (settingsOpen = !settingsOpen)}
        aria-label="Assistant settings"
        title="Assistant settings"
        aria-expanded={settingsOpen}
      >
        <Settings size={16} />
      </button>
      <button
        type="button"
        class="icon-button"
        onclick={removeAllChats}
        disabled={chats.length === 0 || busy || historyBusy}
        aria-label="Remove all chats"
        title="Remove all chats"
      >
        <Trash2 size={16} />
      </button>
    </div>
  </header>

  {#if settingsOpen}
    <div class="assistant-settings shrink-0 border-b border-surface-700 px-3 py-3">
      <div class="settings-grid">
        <label class="field-label">
          <span>Model</span>
          <div class="flex min-w-0 gap-1">
            <select class="assistant-select min-w-0 flex-1" bind:value={settings.model} disabled={!authenticated}>
              {#if settings.model && !models.includes(settings.model)}
                <option value={settings.model}>{settings.model}</option>
              {/if}
              {#each models as model}
                <option value={model}>{model}</option>
              {/each}
            </select>
            <button
              type="button"
              class="icon-button field-icon"
              onclick={() => refreshModels()}
              disabled={!authenticated || loadingModels}
              aria-label="Refresh models"
              title="Refresh models"
            >
              <span class:spin={loadingModels}><RefreshCw size={15} /></span>
            </button>
          </div>
        </label>

        <fieldset class="field-label">
          <legend>Reasoning</legend>
          <div class="reasoning-control" aria-label="Reasoning effort">
            {#each ["low", "medium", "high", "xhigh"] as effort}
              <button
                type="button"
                class:active={settings.reasoning_effort === effort}
                onclick={() => (settings = { ...settings, reasoning_effort: effort })}
              >
                {effort === "xhigh" ? "X-High" : effort[0].toUpperCase() + effort.slice(1)}
              </button>
            {/each}
          </div>
        </fieldset>

        <label class="field-label proxy-field">
          <span>Proxy URL</span>
          <input
            class="assistant-input"
            type="url"
            placeholder="System proxy"
            bind:value={settings.proxy_url}
            disabled={settings.bypass_proxy}
          />
        </label>

        <div class="toggle-group">
          <label class="toggle-label">
            <input type="checkbox" bind:checked={settings.bypass_proxy} />
            <span>Bypass proxy</span>
          </label>
          <label class="toggle-label warning-toggle">
            <input
              type="checkbox"
              checked={settings.disable_tls}
              onchange={handleTlsBypassChange}
            />
            <span>Disable TLS verification</span>
          </label>
        </div>
      </div>
      <div class="mt-2 flex items-center justify-end gap-2">
        <span class="mr-auto text-[11px] text-warning-300">
          Experimental, unofficial Codex compatibility
        </span>
        {#if settingsSaved}
          <span class="flex items-center gap-1 text-xs text-success-300"><Check size={13} />Saved</span>
        {/if}
        <button type="button" class="save-button" onclick={() => saveSettings()}>
          <Save size={14} />
          Save
        </button>
      </div>
    </div>
  {/if}

  {#if errorMessage}
    <div class="error-banner shrink-0 border-b border-error-700/60 px-3 py-2 text-xs text-error-200">
      {errorMessage}
    </div>
  {/if}

  <div class="assistant-body flex min-h-0 flex-1">
    {#if sidebarOpen}
      <aside class="chat-sidebar flex min-h-0 shrink-0 flex-col border-r border-surface-700" aria-label="Chats">
        <div class="chat-sidebar-header flex h-9 shrink-0 items-center border-b border-surface-700 px-2">
          <span class="text-[11px] font-semibold uppercase text-surface-500">Chats</span>
          <div class="ml-auto flex items-center gap-1">
            <button type="button" class="icon-button compact-icon" onclick={createChat} disabled={busy || historyBusy} aria-label="New chat" title="New chat">
              <Plus size={14} />
            </button>
            <button
              type="button"
              class="icon-button compact-icon"
              onclick={removeAllChats}
              disabled={busy || historyBusy || chats.length === 0}
              aria-label="Remove all chats"
              title="Remove all chats"
            >
              <Trash2 size={14} />
            </button>
          </div>
        </div>
        <div class="chat-list min-h-0 flex-1 overflow-y-auto p-1.5">
          {#each orderedChats as chat (chat.id)}
            <div class:active={chat.id === activeChatId} class="chat-row flex min-w-0 items-center">
              <button
                type="button"
                class="chat-select flex min-w-0 flex-1 items-start gap-2 text-left"
                onclick={() => selectChat(chat.id)}
                disabled={busy || historyBusy}
                aria-current={chat.id === activeChatId ? "page" : undefined}
              >
                <span class="chat-icon mt-0.5 shrink-0"><MessageSquare size={14} /></span>
                <span class="min-w-0 flex-1">
                  <span class="chat-title block truncate">{chat.title}</span>
                  <span class="chat-meta block truncate">{chat.messages.length} messages · {formatChatTime(chat.updatedAt)}</span>
                </span>
              </button>
              <button
                type="button"
                class="chat-delete"
                onclick={(event) => removeChat(event, chat.id)}
                disabled={busy || historyBusy}
                aria-label={`Remove ${chat.title}`}
                title="Remove chat"
              >
                <Trash2 size={13} />
              </button>
            </div>
          {:else}
            <div class="chat-list-empty flex h-24 items-center justify-center text-xs text-surface-600">No chats</div>
          {/each}
        </div>
      </aside>
    {/if}

    <div class="conversation-shell flex min-w-0 flex-1 flex-col">
      <div class="message-scroll min-h-0 flex-1 overflow-y-auto" bind:this={messagesContainer}>
        {#if !activeChatId}
          <div class="empty-state flex min-h-full flex-col items-center justify-center gap-4 px-6 py-8 text-center">
            <MessageSquarePlus size={30} strokeWidth={1.25} class="text-surface-500" />
            <button type="button" class="connect-button" onclick={createChat} disabled={historyBusy}><Plus size={15} />New chat</button>
          </div>
        {:else if messages.length === 0}
          <div class="empty-state flex min-h-full flex-col items-center justify-center gap-4 px-6 py-8 text-center">
            <Bot size={30} strokeWidth={1.25} class="text-surface-500" />
            {#if authenticated}
              <div class="prompt-actions">
                <button type="button" onclick={() => usePrompt("Explain the currently selected item.")}>Explain selection</button>
                <button type="button" onclick={() => usePrompt("Trace references to and from the currently selected item.")}>Trace references</button>
                <button type="button" onclick={() => usePrompt("Review the currently selected function for security risks.")}>Review security</button>
              </div>
            {:else}
              <button type="button" class="connect-button" onclick={connectChatGPT} disabled={authenticating}>
                <LogIn size={15} />
                {authenticating ? "Waiting for browser" : "Connect ChatGPT"}
              </button>
            {/if}
          </div>
        {:else}
          <div class="message-list mx-auto flex w-full max-w-4xl flex-col py-2">
            {#each messages as message, messageIndex}
              <article class:user-message={message.role === "user"} class="message-row px-4 py-3">
                <div class="mb-1.5 flex items-center gap-2 text-xs font-medium text-surface-400">
                  {#if message.role === "assistant"}
                    <Bot size={14} />
                    <span>{message.model || "Assistant"}</span>
                    {@const messageKey = `${activeChatId}:${messageIndex}`}
                    <button
                      type="button"
                      class:copied={copiedMessageKey === messageKey}
                      class="copy-button ml-auto"
                      onclick={() => copyAnswer(message.content, messageKey)}
                      aria-label={copiedMessageKey === messageKey ? "Answer copied" : "Copy answer"}
                      title={copiedMessageKey === messageKey ? "Copied" : "Copy answer"}
                    >
                      {#if copiedMessageKey === messageKey}<Check size={14} />{:else}<Copy size={14} />{/if}
                    </button>
                  {:else}
                    <span>You</span>
                  {/if}
                </div>
                {#if message.reasoning && message.reasoning.length > 0}
                  <details class="reasoning-trace mb-3">
                    <summary class="flex cursor-pointer items-center gap-2 text-xs text-surface-400">
                      <Brain size={13} />
                      <span>Thinking</span>
                      <span class="trace-count">{message.reasoning.length}</span>
                      <span class="trace-chevron ml-auto"><ChevronDown size={13} /></span>
                    </summary>
                    <div class="reasoning-parts mt-2">
                      {#each message.reasoning as part, partIndex}
                        <div class="reasoning-part">
                          <span>Step {partIndex + 1}</span>
                          <p>{part}</p>
                        </div>
                      {/each}
                    </div>
                  </details>
                {/if}
                {#if message.role === "assistant"}
                  <div class="markdown-content text-sm leading-6 text-surface-100">{@html renderMarkdown(message.content)}</div>
                {:else}
                  <div class="message-content text-sm leading-6 text-surface-100">{message.content}</div>
                {/if}
                {#if message.toolActivity && message.toolActivity.length > 0}
                  <details class="tool-trace mt-3">
                    <summary class="flex cursor-pointer items-center gap-2 text-xs text-surface-400">
                      <Wrench size={13} />
                      <span>{message.toolActivity.length} tool {message.toolActivity.length === 1 ? "call" : "calls"}</span>
                      <span class="trace-chevron ml-auto"><ChevronDown size={13} /></span>
                    </summary>
                    <div class="mt-2 divide-y divide-surface-700/70 border-y border-surface-700/70">
                      {#each message.toolActivity as activity}
                        <div class="tool-entry py-2 text-xs">
                          <div class="flex items-center gap-2">
                            <span class:tool-failed={!activity.success} class="tool-status"></span>
                            <code class="text-secondary-200">{displayToolName(activity.name)}</code>
                          </div>
                          <p class="tool-summary mt-1 pl-4 leading-5 text-surface-400">{activity.summary}</p>
                        </div>
                      {/each}
                    </div>
                  </details>
                {/if}
                {#if message.role === "assistant"}
                  {@const footerMessageKey = `${activeChatId}:${messageIndex}`}
                  <div class="answer-footer mt-2 flex min-h-7 items-center gap-2">
                    {#if message.usage}
                      <span class="text-[11px] text-surface-500">{message.usage.totalTokens.toLocaleString()} tokens</span>
                    {/if}
                    <button
                      type="button"
                      class:copied={copiedMessageKey === footerMessageKey}
                      class="footer-copy-button ml-auto"
                      onclick={() => copyAnswer(message.content, footerMessageKey)}
                      aria-label={copiedMessageKey === footerMessageKey ? "Answer copied" : "Copy raw answer"}
                      title={copiedMessageKey === footerMessageKey ? "Copied" : "Copy raw answer"}
                    >
                      {#if copiedMessageKey === footerMessageKey}<Check size={13} />{:else}<Copy size={13} />{/if}
                      <span>{copiedMessageKey === footerMessageKey ? "Copied" : "Copy"}</span>
                    </button>
                  </div>
                {/if}
              </article>
            {/each}
            {#if busy}
              <article class="message-row processing-row px-4 py-3">
                <div class="flex items-center gap-2 text-xs text-surface-400">
                  <LoaderCircle size={14} class="spin" />
                  <span>
                    {liveFinalizing ? "Preparing answer" : liveToolName ? `Using ${displayToolName(liveToolName)}` : "Thinking"}
                  </span>
                  {#if liveRound > 0}<span class="ml-auto text-[10px] text-surface-600">Round {liveRound}</span>{/if}
                </div>
                {#if liveReasoning.length > 0}
                  <details class="reasoning-trace live-trace mt-3" open>
                    <summary class="flex cursor-pointer items-center gap-2 text-xs text-surface-400">
                      <Brain size={13} />
                      <span>Thinking</span>
                      <span class="trace-count">{liveReasoning.length}</span>
                      <span class="trace-chevron ml-auto"><ChevronDown size={13} /></span>
                    </summary>
                    <div class="reasoning-parts mt-2">
                      {#each liveReasoning as part, partIndex}
                        <div class="reasoning-part">
                          <span>Step {partIndex + 1}</span>
                          <p>{part}</p>
                        </div>
                      {/each}
                    </div>
                  </details>
                {/if}
                {#if liveOutput}
                  <div class="message-content mt-3 text-sm leading-6 text-surface-100">{liveOutput}</div>
                {/if}
                {#if liveToolActivity.length > 0 || liveToolName}
                  <details class="tool-trace live-trace mt-3" open>
                    <summary class="flex cursor-pointer items-center gap-2 text-xs text-surface-400">
                      <Wrench size={13} />
                      <span>{liveToolActivity.length} completed</span>
                      {#if liveToolName}<span class="truncate text-secondary-200">{displayToolName(liveToolName)}</span>{/if}
                      <span class="trace-chevron ml-auto"><ChevronDown size={13} /></span>
                    </summary>
                    <div class="mt-2 divide-y divide-surface-700/70 border-y border-surface-700/70">
                      {#each liveToolActivity as activity}
                        <div class="tool-entry py-2 text-xs">
                          <div class="flex items-center gap-2">
                            <span class:tool-failed={!activity.success} class="tool-status"></span>
                            <code class="text-secondary-200">{displayToolName(activity.name)}</code>
                          </div>
                          <p class="tool-summary mt-1 pl-4 leading-5 text-surface-400">{activity.summary}</p>
                        </div>
                      {/each}
                    </div>
                  </details>
                {/if}
                {#if !liveOutput && liveReasoning.length === 0 && liveToolActivity.length === 0 && !liveToolName}
                  <div class="thinking-line mt-3"><span></span><span></span><span></span></div>
                {/if}
              </article>
            {/if}
          </div>
        {/if}
      </div>

      <footer class="composer-wrap shrink-0 border-t border-surface-700 p-2">
        <div class="composer mx-auto flex w-full max-w-4xl items-end gap-2">
          <textarea
            bind:value={prompt}
            onkeydown={handlePromptKeydown}
            rows="2"
            placeholder={!authenticated ? "Connect ChatGPT to continue" : activeChatId ? "Ask about the loaded bytecode" : "Create a chat to continue"}
            disabled={!authenticated || !activeChatId || busy}
            aria-label="Assistant message"
          ></textarea>
          {#if busy}
            <button type="button" class="send-button stop-button" onclick={cancelRequest} aria-label="Stop response" title="Stop response">
              <Square size={15} fill="currentColor" />
            </button>
          {:else}
            <button
              type="button"
              class="send-button"
              onclick={submitMessage}
              disabled={!authenticated || !activeChatId || !prompt.trim()}
              aria-label="Send message"
              title="Send message"
            >
              <Send size={16} />
            </button>
          {/if}
        </div>
      </footer>
    </div>
  </div>
</section>

{#if privacyDisclosureOpen}
  <div class="privacy-overlay" role="presentation">
    <div
      class="privacy-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="assistant-privacy-title"
    >
      <header class="privacy-header">
        <ShieldCheck size={20} aria-hidden="true" />
        <h2 id="assistant-privacy-title">Assistant privacy disclosure</h2>
      </header>
      <p>Before connecting or sending a request:</p>
      <ul>
        <li>Your messages are sent to ChatGPT.</li>
        <li>
          Selected bytecode metadata and any requested decompilation, disassembly, or tool output
          may also be sent to ChatGPT.
        </li>
        {#if encryptedPersistenceAvailable}
          <li>
            Chat history is stored locally as authenticated ciphertext; its random encryption key
            is kept in the operating-system credential vault.
          </li>
        {:else}
          <li>
            Encrypted persistence is unavailable, so chats created in this session are session-only
            and are not written to disk.
          </li>
        {/if}
        <li>
          A configured external HTTP helper receives OAuth headers and request bodies through its
          standard input. It must be a fully trusted executable.
        </li>
        <li>Custom proxies can observe connection metadata.</li>
        <li>
          Disabling TLS verification permits interception of credentials and analyzed content.
        </li>
      </ul>
      <div class="privacy-actions">
        <button type="button" class="privacy-secondary" onclick={() => (privacyDisclosureOpen = false)}>
          Not now
        </button>
        <button
          type="button"
          class="save-button"
          onclick={acceptPrivacyDisclosure}
          disabled={acceptingPrivacy}
        >
          <Check size={14} />
          {acceptingPrivacy ? "Saving" : "Accept and continue"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .assistant-view {
    background: color-mix(in srgb, var(--color-surface-950) 96%, black);
    color: var(--color-surface-100);
  }

  .privacy-overlay {
    position: fixed;
    z-index: 1000;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 72%);
    padding: 1rem;
  }

  .privacy-dialog {
    width: min(34rem, 100%);
    max-height: min(42rem, calc(100vh - 2rem));
    overflow-y: auto;
    border: 1px solid var(--color-surface-600);
    border-radius: 0.375rem;
    background: var(--color-surface-900);
    padding: 1rem;
    color: var(--color-surface-200);
    box-shadow: 0 1rem 3rem rgb(0 0 0 / 48%);
  }

  .privacy-header {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    color: var(--color-surface-50);
  }

  .privacy-header h2 {
    font-size: 0.95rem;
    font-weight: 650;
  }

  .privacy-dialog > p {
    margin-top: 0.8rem;
    color: var(--color-surface-400);
    font-size: 0.78rem;
  }

  .privacy-dialog ul {
    margin: 0.65rem 0 0;
    padding-left: 1.2rem;
    color: var(--color-surface-200);
    font-size: 0.78rem;
    line-height: 1.25rem;
  }

  .privacy-dialog li + li {
    margin-top: 0.35rem;
  }

  .privacy-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .privacy-secondary {
    min-height: 1.85rem;
    border: 1px solid var(--color-surface-600);
    border-radius: 0.25rem;
    padding: 0.25rem 0.65rem;
    color: var(--color-surface-300);
    font-size: 0.75rem;
  }

  .assistant-toolbar,
  .composer-wrap,
  .assistant-settings,
  .chat-sidebar {
    background: color-mix(in srgb, var(--color-surface-900) 88%, black);
  }

  .assistant-body,
  .conversation-shell {
    min-width: 0;
  }

  .chat-sidebar {
    width: min(13rem, 42%);
    min-width: 8.5rem;
  }

  .chat-sidebar-header {
    background: color-mix(in srgb, var(--color-surface-800) 35%, transparent);
  }

  .compact-icon {
    width: 1.55rem;
    height: 1.55rem;
  }

  .chat-list {
    scrollbar-gutter: stable;
  }

  .chat-row {
    min-height: 3.15rem;
    margin-bottom: 0.2rem;
    border: 1px solid transparent;
    border-radius: 0.25rem;
    color: var(--color-surface-400);
  }

  .chat-row:hover {
    border-color: color-mix(in srgb, var(--color-surface-600) 72%, transparent);
    background: color-mix(in srgb, var(--color-surface-800) 62%, transparent);
  }

  .chat-row.active {
    border-color: color-mix(in srgb, var(--color-secondary-600) 62%, var(--color-surface-700));
    background: color-mix(in srgb, var(--color-secondary-700) 17%, var(--color-surface-800));
    color: var(--color-surface-100);
  }

  .chat-select {
    min-height: 3rem;
    padding: 0.48rem 0.25rem 0.4rem 0.5rem;
  }

  .chat-select:disabled,
  .chat-delete:disabled {
    cursor: not-allowed;
  }

  .chat-icon {
    color: var(--color-surface-500);
  }

  .chat-row.active .chat-icon {
    color: var(--color-secondary-300);
  }

  .chat-title {
    font-size: 0.72rem;
    font-weight: 600;
    line-height: 1rem;
  }

  .chat-meta {
    margin-top: 0.12rem;
    color: var(--color-surface-600);
    font-size: 0.61rem;
    line-height: 0.85rem;
  }

  .chat-row.active .chat-meta {
    color: var(--color-surface-400);
  }

  .chat-delete {
    display: inline-flex;
    width: 1.6rem;
    height: 1.8rem;
    flex: 0 0 auto;
    align-items: center;
    justify-content: center;
    border-radius: 0.2rem;
    color: var(--color-surface-600);
    opacity: 0;
  }

  .chat-row:hover .chat-delete,
  .chat-row.active .chat-delete,
  .chat-delete:focus-visible {
    opacity: 1;
  }

  .chat-delete:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-error-800) 42%, transparent);
    color: var(--color-error-300);
  }

  .connection-dot,
  .tool-status {
    width: 0.45rem;
    height: 0.45rem;
    flex: 0 0 auto;
    border-radius: 999px;
    background: var(--color-surface-600);
  }

  .connection-dot.connected,
  .tool-status {
    background: var(--color-success-500);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-success-500) 18%, transparent);
  }

  .tool-status.tool-failed {
    background: var(--color-error-500);
    box-shadow: none;
  }

  .icon-button,
  .send-button {
    display: inline-flex;
    width: 1.85rem;
    height: 1.85rem;
    flex: 0 0 auto;
    align-items: center;
    justify-content: center;
    border: 1px solid transparent;
    border-radius: 0.25rem;
    color: var(--color-surface-300);
  }

  .icon-button:hover:not(:disabled) {
    border-color: var(--color-surface-600);
    background: var(--color-surface-800);
    color: var(--color-surface-50);
  }

  .icon-button:disabled,
  .send-button:disabled {
    cursor: not-allowed;
    opacity: 0.38;
  }

  .connect-button,
  .save-button {
    display: inline-flex;
    min-height: 1.85rem;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    border: 1px solid color-mix(in srgb, var(--color-secondary-500) 62%, var(--color-surface-600));
    border-radius: 0.25rem;
    background: color-mix(in srgb, var(--color-secondary-600) 38%, var(--color-surface-800));
    padding: 0.25rem 0.65rem;
    color: var(--color-surface-50);
    font-size: 0.75rem;
    font-weight: 600;
  }

  .connect-button:hover:not(:disabled),
  .save-button:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-secondary-500) 52%, var(--color-surface-800));
  }

  .settings-grid {
    display: grid;
    grid-template-columns: minmax(10rem, 1.2fr) minmax(14rem, 1.4fr);
    gap: 0.65rem 1rem;
  }

  .field-label,
  .toggle-group {
    min-width: 0;
    color: var(--color-surface-400);
    font-size: 0.7rem;
  }

  .field-label > span,
  .field-label > legend {
    display: block;
    margin-bottom: 0.3rem;
  }

  .assistant-select,
  .assistant-input {
    height: 1.85rem;
    min-width: 0;
    border: 1px solid var(--color-surface-700);
    border-radius: 0.25rem;
    background: var(--color-surface-950);
    padding: 0 0.5rem;
    color: var(--color-surface-100);
    font-size: 0.75rem;
    outline: none;
  }

  .assistant-select:focus,
  .assistant-input:focus,
  .composer textarea:focus {
    border-color: var(--color-primary-500);
  }

  .field-icon {
    border-color: var(--color-surface-700);
  }

  .reasoning-control {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    height: 1.85rem;
    overflow: hidden;
    border: 1px solid var(--color-surface-700);
    border-radius: 0.25rem;
  }

  .reasoning-control button {
    min-width: 0;
    border-right: 1px solid var(--color-surface-700);
    color: var(--color-surface-400);
    font-size: 0.68rem;
  }

  .reasoning-control button:last-child {
    border-right: 0;
  }

  .reasoning-control button.active {
    background: color-mix(in srgb, var(--color-secondary-600) 30%, var(--color-surface-800));
    color: var(--color-secondary-100);
  }

  .toggle-group {
    display: flex;
    min-height: 1.85rem;
    align-items: end;
    gap: 1rem;
    padding-bottom: 0.2rem;
  }

  .toggle-label {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    white-space: nowrap;
  }

  .toggle-label input {
    accent-color: var(--color-primary-500);
  }

  .warning-toggle {
    color: var(--color-warning-300);
  }

  .message-scroll {
    scrollbar-gutter: stable;
  }

  .empty-state {
    color: var(--color-surface-500);
  }

  .prompt-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 0.5rem;
  }

  .prompt-actions button {
    min-height: 2rem;
    border: 1px solid var(--color-surface-700);
    border-radius: 0.25rem;
    background: var(--color-surface-900);
    padding: 0.35rem 0.7rem;
    color: var(--color-surface-300);
    font-size: 0.75rem;
  }

  .prompt-actions button:hover {
    border-color: var(--color-secondary-600);
    color: var(--color-surface-100);
  }

  .message-row {
    border-bottom: 1px solid color-mix(in srgb, var(--color-surface-700) 52%, transparent);
  }

  .message-row.user-message {
    background: color-mix(in srgb, var(--color-surface-800) 34%, transparent);
  }

  .message-content {
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }

  .copy-button {
    display: inline-flex;
    width: 1.75rem;
    height: 1.75rem;
    flex: 0 0 1.75rem;
    align-items: center;
    justify-content: center;
    border: 1px solid transparent;
    border-radius: 0.25rem;
    color: var(--color-surface-500);
  }

  .copy-button:hover,
  .copy-button:focus-visible {
    border-color: var(--color-surface-700);
    background: var(--color-surface-800);
    color: var(--color-surface-200);
    outline: none;
  }

  .copy-button.copied {
    color: var(--color-success-400);
  }

  .footer-copy-button {
    display: inline-flex;
    min-height: 1.75rem;
    flex: 0 0 auto;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    border: 1px solid var(--color-surface-700);
    border-radius: 0.25rem;
    padding: 0.2rem 0.5rem;
    color: var(--color-surface-400);
    font-size: 0.7rem;
    line-height: 1rem;
  }

  .footer-copy-button:hover,
  .footer-copy-button:focus-visible {
    background: var(--color-surface-800);
    color: var(--color-surface-100);
    outline: none;
  }

  .footer-copy-button.copied {
    border-color: color-mix(in srgb, var(--color-success-500) 45%, var(--color-surface-700));
    color: var(--color-success-400);
  }

  .markdown-content {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .markdown-content :global(> :first-child) {
    margin-top: 0;
  }

  .markdown-content :global(> :last-child) {
    margin-bottom: 0;
  }

  .markdown-content :global(p),
  .markdown-content :global(ul),
  .markdown-content :global(ol),
  .markdown-content :global(blockquote),
  .markdown-content :global(pre),
  .markdown-content :global(table) {
    margin: 0.65rem 0;
  }

  .markdown-content :global(h1),
  .markdown-content :global(h2),
  .markdown-content :global(h3),
  .markdown-content :global(h4),
  .markdown-content :global(h5),
  .markdown-content :global(h6) {
    margin: 1rem 0 0.45rem;
    color: var(--color-surface-50);
    font-weight: 650;
    line-height: 1.35;
  }

  .markdown-content :global(h1) {
    font-size: 1.2rem;
  }

  .markdown-content :global(h2) {
    font-size: 1.05rem;
  }

  .markdown-content :global(h3),
  .markdown-content :global(h4),
  .markdown-content :global(h5),
  .markdown-content :global(h6) {
    font-size: 0.95rem;
  }

  .markdown-content :global(ul),
  .markdown-content :global(ol) {
    padding-left: 1.4rem;
  }

  .markdown-content :global(ul) {
    list-style: disc;
  }

  .markdown-content :global(ol) {
    list-style: decimal;
  }

  .markdown-content :global(li + li) {
    margin-top: 0.2rem;
  }

  .markdown-content :global(a) {
    color: var(--color-secondary-300);
    text-decoration: underline;
    text-underline-offset: 0.16rem;
  }

  .markdown-content :global(a:hover) {
    color: var(--color-secondary-200);
  }

  .markdown-content :global(blockquote) {
    border-left: 3px solid var(--color-surface-600);
    padding-left: 0.8rem;
    color: var(--color-surface-300);
  }

  .markdown-content :global(code) {
    border-radius: 0.2rem;
    background: var(--color-surface-800);
    padding: 0.12rem 0.3rem;
    color: var(--color-secondary-100);
    font-size: 0.78rem;
  }

  .markdown-content :global(pre) {
    max-width: 100%;
    overflow-x: auto;
    border: 1px solid var(--color-surface-700);
    border-radius: 0.25rem;
    background: color-mix(in srgb, var(--color-surface-950) 88%, black);
    padding: 0.7rem 0.8rem;
  }

  .markdown-content :global(pre code) {
    border-radius: 0;
    background: transparent;
    padding: 0;
    color: var(--color-surface-100);
    white-space: pre;
  }

  .markdown-content :global(table) {
    display: block;
    max-width: 100%;
    overflow-x: auto;
    border-collapse: collapse;
    font-size: 0.78rem;
  }

  .markdown-content :global(th),
  .markdown-content :global(td) {
    border: 1px solid var(--color-surface-700);
    padding: 0.35rem 0.55rem;
    text-align: left;
    white-space: nowrap;
  }

  .markdown-content :global(th) {
    background: var(--color-surface-800);
    color: var(--color-surface-100);
  }

  .markdown-content :global(hr) {
    margin: 1rem 0;
    border: 0;
    border-top: 1px solid var(--color-surface-700);
  }

  .tool-trace,
  .reasoning-trace {
    width: 100%;
    min-width: 0;
    max-width: 46rem;
    border: 1px solid var(--color-surface-700);
    border-radius: 0.25rem;
    background: color-mix(in srgb, var(--color-surface-900) 74%, transparent);
    padding: 0.5rem 0.65rem;
  }

  .tool-trace[open] .trace-chevron,
  .reasoning-trace[open] .trace-chevron {
    transform: rotate(180deg);
  }

  .reasoning-trace {
    background: color-mix(in srgb, var(--color-surface-900) 82%, transparent);
  }

  .tool-entry,
  .tool-entry > div {
    min-width: 0;
  }

  .tool-entry code,
  .tool-summary {
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }

  .reasoning-trace.live-trace,
  .tool-trace.live-trace {
    border-color: color-mix(in srgb, var(--color-secondary-600) 46%, var(--color-surface-700));
  }

  .trace-count {
    min-width: 1rem;
    border-radius: 999px;
    background: var(--color-surface-800);
    padding: 0.05rem 0.3rem;
    color: var(--color-surface-500);
    text-align: center;
    font-size: 0.62rem;
  }

  .reasoning-parts {
    border-top: 1px solid color-mix(in srgb, var(--color-surface-700) 72%, transparent);
  }

  .reasoning-part {
    padding: 0.55rem 0 0.15rem;
  }

  .reasoning-part + .reasoning-part {
    margin-top: 0.35rem;
    border-top: 1px solid color-mix(in srgb, var(--color-surface-700) 52%, transparent);
  }

  .reasoning-part > span {
    color: var(--color-secondary-300);
    font-size: 0.62rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .reasoning-part p {
    margin-top: 0.2rem;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
    color: var(--color-surface-300);
    font-size: 0.75rem;
    line-height: 1.2rem;
  }

  .trace-chevron {
    transition: transform 120ms ease;
  }

  .error-banner {
    overflow-wrap: anywhere;
    background: color-mix(in srgb, var(--color-error-900) 32%, var(--color-surface-900));
  }

  .composer {
    min-height: 3.6rem;
  }

  .composer textarea {
    min-height: 3.35rem;
    max-height: 8rem;
    flex: 1;
    resize: vertical;
    border: 1px solid var(--color-surface-700);
    border-radius: 0.25rem;
    background: var(--color-surface-950);
    padding: 0.55rem 0.65rem;
    color: var(--color-surface-100);
    font-size: 0.8rem;
    line-height: 1.15rem;
    outline: none;
  }

  .composer textarea::placeholder {
    color: var(--color-surface-600);
  }

  .send-button {
    width: 2rem;
    height: 2rem;
    margin-bottom: 0.1rem;
    border-color: var(--color-secondary-600);
    background: var(--color-secondary-600);
    color: var(--color-secondary-contrast-600);
  }

  .send-button:hover:not(:disabled) {
    background: var(--color-secondary-500);
  }

  .stop-button {
    border-color: var(--color-error-600);
    background: var(--color-error-700);
    color: white;
  }

  .thinking-line {
    display: flex;
    height: 1rem;
    align-items: center;
    gap: 0.25rem;
  }

  .thinking-line span {
    width: 0.3rem;
    height: 0.3rem;
    border-radius: 999px;
    background: var(--color-secondary-400);
    animation: pulse 1.2s ease-in-out infinite;
  }

  .thinking-line span:nth-child(2) { animation-delay: 120ms; }
  .thinking-line span:nth-child(3) { animation-delay: 240ms; }

  .spin { animation: spin 900ms linear infinite; }

  @keyframes pulse {
    0%, 70%, 100% { opacity: 0.25; transform: translateY(0); }
    35% { opacity: 1; transform: translateY(-2px); }
  }

  @keyframes spin { to { transform: rotate(360deg); } }

  @media (max-width: 700px) {
    .settings-grid { grid-template-columns: 1fr; }
    .toggle-group { align-items: flex-start; flex-direction: column; gap: 0.45rem; }
    .assistant-toolbar { padding-inline: 0.5rem; }
    .message-row { padding-inline: 0.75rem; }
    .chat-sidebar { min-width: 7.75rem; }
    .chat-select { padding-left: 0.35rem; }
    .chat-meta { display: none; }
    .assistant-toolbar .connect-button {
      width: 1.85rem;
      min-height: 1.85rem;
      padding: 0;
    }
    .assistant-toolbar .connect-label { display: none; }
  }

  @media (max-width: 520px) {
    .model-label,
    .connection-dot { display: none; }
    .assistant-toolbar { gap: 0.3rem; }
  }

  @media (prefers-reduced-motion: reduce) {
    .thinking-line span,
    .spin { animation: none; }
    .message-scroll { scroll-behavior: auto; }
  }
</style>
