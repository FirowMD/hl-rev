<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import hljs from 'highlight.js/lib/core';
  import haxe from 'highlight.js/lib/languages/haxe';
  import 'highlight.js/styles/github-dark.css';
  import { writable } from 'svelte/store';
  import type { AIDecompilation, BytecodeItemSelectedEvent } from './types';
  import VirtualList from 'svelte-tiny-virtual-list';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { getContext } from 'svelte';

  hljs.registerLanguage('haxe', haxe);

  let openrouterKey = $state("");
  let selectedModel = $state("deepseek/deepseek-r1:free");
  let customPrompt = $state("Decompile please following hashlink disassembly. Provide only decompiled code, no other words.");
  let disassemblyCode = "";
  let decompilationResult = $state("");
  let isLoading = $state(false);
  let currentFunction = $state("");
  let showSettings = $state(false);
  let showReplaced = $state(false);
  let replacedSearch = $state("");
  let replacedItems = $state<AIDecompilation[]>([]);
  let showEditor = $state(false);

  const replacedFunctions = writable<Set<string>>(new Set());

  const models = [
    "deepseek/deepseek-r1:free",
    "google/gemini-2.0-pro-exp-02-05:free",
    "google/gemini-2.0-flash-exp:free",
    "google/gemini-2.0-flash-lite-preview-02-05:free",
    "meta-llama/llama-3.3-70b-instruct:free",
    "qwen/qwq-32b:free"
  ];

  async function loadConfig() {
    openrouterKey = await invoke("get_config_openrouter_key");
    selectedModel = await invoke("get_config_ai_decompiler");
    const savedPrompt = await invoke("get_config_prompt") as string;
    if (savedPrompt) {
      customPrompt = savedPrompt;
    }
  }

  async function saveConfig() {
    await invoke("set_config_openrouter_key", { key: openrouterKey });
    await invoke("set_config_ai_decompiler", { model: selectedModel });
    await invoke("set_config_prompt", { prompt: customPrompt });
    await invoke("save_config");
  }

  async function checkAndLoadSelectedItem() {
    try {
      const item = await invoke("get_selected_item") as { typ: string, index: string } | null;
      if (item && item.typ === "function") {
        const index = parseInt(item.index);
        currentFunction = await invoke("get_function_name_by_index", { index }) as string;
        disassemblyCode = await invoke("get_disassembler_info") as string;
      }
      console.log("Selected item:", item);
    } catch (error) {
      console.error("Error loading selected item:", error);
    }
  }

  async function decompileCode() {
    if (!openrouterKey) {
      alert("Please enter your OpenRouter API key");
      return;
    }

    isLoading = true;
    try {
      const requestBody = {
        model: selectedModel,
        messages: [
          {
            role: 'user',
            content: `${customPrompt}\n\n${disassemblyCode}`,
          },
        ],
      };

      const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${openrouterKey}`,
          'Content-Type': 'application/json',
          'HTTP-Referer': window.location.origin,
          'X-Title': 'ByteSto4t',
        },
        body: JSON.stringify(requestBody),
      });

      const data = await response.json();

      if (!data.choices?.[0]?.message?.content) {
        throw new Error("Invalid response format: " + JSON.stringify(data));
      }

      decompilationResult = data.choices[0].message.content;
    } catch (error) {
      console.error("Decompilation failed:", error);
      alert("Decompilation failed. Please check your API key and try again.");
      decompilationResult = "";
    } finally {
      isLoading = false;
    }
  }

  async function saveDecompilation() {
    if (!currentFunction || !decompilationResult) return;
    
    try {
      await invoke("save_ai_decompilation", {
        functionName: currentFunction,
        result: decompilationResult,
        model: selectedModel
      });
      
      replacedFunctions.update(set => {
        set.add(currentFunction);
        return set;
      });

      await loadReplacedItems();

      window.dispatchEvent(new CustomEvent('ai-decompilation-replaced', {
        detail: {
          functionName: currentFunction,
          result: decompilationResult
        }
      }));

      await invoke("save_config");
      alert("Successfully replaced original decompilation!");
    } catch (error) {
      console.error("Failed to save decompilation:", error);
      alert("Failed to save decompilation");
    }
  }

  async function loadReplacedItems() {
    try {
      const items = await invoke("get_ai_decompilations") as AIDecompilation[];
      replacedItems = items;
    } catch (error) {
      console.error("Failed to load replaced items:", error);
    }
  }

  async function removeDecompilation(functionName: string) {
    try {
      await invoke("remove_ai_decompilation", { functionName });
      await loadReplacedItems();
      
      replacedFunctions.update(set => {
        set.delete(functionName);
        return set;
      });

      window.dispatchEvent(new CustomEvent('ai-decompilation-removed', {
        detail: { functionName }
      }));

      await invoke("save_config");
    } catch (error) {
      console.error("Failed to remove decompilation:", error);
      alert("Failed to remove decompilation");
    }
  }

  function formatTimestamp(isoString: string): string {
    return new Date(isoString).toLocaleString();
  }

  onMount(() => {
    loadConfig();
    checkAndLoadSelectedItem();
    
    window.addEventListener("bytecode-item-selected", async (e: Event) => {
      const ev = e as CustomEvent<BytecodeItemSelectedEvent>;
      if (ev.detail.type === "function") {
        currentFunction = ev.detail.name;
        disassemblyCode = await invoke("get_disassembler_info") as string;
      }
    });
  });

  $effect(() => {
    checkAndLoadSelectedItem();
  });

  $effect(() => {
    invoke("update_replaced_decompilations")
      .then(() => loadReplacedItems())
      .catch(error => console.error("Failed to update decompilations:", error));
  });

  const filteredReplacedItems = $derived(replacedItems.filter(item => 
    item.function_name.toLowerCase().includes(replacedSearch.toLowerCase())
  ));
</script>

<div class="h-full preset-outlined-surface-500 bg-surface-900 overflow-y-auto">
  <div class="p-2 space-y-2  h-full">
    <section class="space-y-2">
      <div class="flex justify-between items-center">
        <h5 class="h5">AI Decompiler</h5>
        <button 
          type="button" 
          class="btn preset-filled-surface-500"
          onclick={() => showSettings = !showSettings}
        >
          {showSettings ? 'Hide Settings' : 'Show Settings'}
        </button>
      </div>

      {#if showSettings}
        <div class="space-y-2">
          <input 
            type="password" 
            class="input w-full focus:outline-none" 
            placeholder="OpenRouter API Key"
            bind:value={openrouterKey}
            onchange={saveConfig}
          />
          <select 
            class="select w-full focus:outline-none" 
            bind:value={selectedModel}
            onchange={saveConfig}
          >
            {#each models as model}
              <option value={model}>{model}</option>
            {/each}
          </select>
          <div class="space-y-1">
            <p class="text-sm">Custom Prompt</p>
            <textarea
              class="textarea w-full focus:outline-none"
              placeholder="Enter custom prompt for the AI"
              rows="3"
              bind:value={customPrompt}
              onchange={saveConfig}
            ></textarea>
          </div>
        </div>
      {/if}

      <div class="flex flex-row space-x-2 items-center">
        <input 
          type="text" 
          class="input bg-surface-800 flex-1 focus:outline-none" 
          placeholder="No function selected"
          value={currentFunction}
          disabled
        />
        <button 
          type="button" 
          class="btn preset-filled-surface-500" 
          onclick={decompileCode}
          disabled={isLoading || !currentFunction}
        >
          {isLoading ? 'Decompiling...' : 'Decompile'}
        </button>
      </div>
    </section>

    {#if decompilationResult}
      <section class="p-4 bg-surface-800">
        <div class="flex justify-between items-center mb-2">
          <div class="text-sm">Result length: {decompilationResult.length}</div>
          <div class="flex gap-2">
            <button 
              type="button" 
              class="btn preset-filled-surface-500"
              onclick={() => showEditor = !showEditor}
            >
              {showEditor ? 'Hide Editor' : 'Edit'}
            </button>
            <button 
              type="button" 
              class="btn preset-filled-surface-500"
              onclick={saveDecompilation}
            >
              Replace Original
            </button>
          </div>
        </div>
        {#if showEditor}
          <textarea
            class="textarea bg-surface-900 w-full font-mono text-sm mb-2 focus:outline-none"
            rows="20"
            bind:value={decompilationResult}
          ></textarea>
        {/if}
        <pre class="bg-surface-700 p-4 rounded-container-token overflow-x-auto"><code class="language-haxe">{decompilationResult}</code></pre>
      </section>
    {/if}

    <section class="space-y-2">
      <div class="flex justify-between items-center">
        <h5 class="h5">Replaced Functions</h5>
        <div class="flex gap-2">
          <button 
            type="button" 
            class="btn preset-filled-surface-500"
            onclick={() => showReplaced = !showReplaced}
          >
            {showReplaced ? 'Hide Replaced' : 'Show Replaced'}
          </button>
          {#if filteredReplacedItems.length > 0}
            <button 
              type="button" 
              class="btn preset-filled-surface-500"
              onclick={async () => {
                const confirmed = await confirm('Are you sure you want to remove all decompilations?');
                if (confirmed) {
                  await invoke('remove_all_decompilations');
                  await loadReplacedItems();
                  replacedFunctions.set(new Set());
                }
              }}
            >
              Remove All
            </button>
          {/if}
        </div>
      </div>

      {#if showReplaced}
        <div class="space-y-2">
          <input 
            type="text" 
            class="input w-full focus:outline-none" 
            placeholder="Search replaced functions..."
            bind:value={replacedSearch}
          />

          <div class="h-96 overflow-y-auto">
            <VirtualList 
              width="100%" 
              height="100%" 
              itemCount={filteredReplacedItems.length} 
              itemSize={80}
              overscanCount={5}
            >
              <div slot="item" let:index let:style {style}>
                <div class="p-2 preset-filled-surface-500 mb-1">
                  <div class="flex justify-between items-center">
                    <div class="flex-1">
                      <div class="font-semibold truncate">
                        {filteredReplacedItems[index].function_name}
                      </div>
                      <div class="text-xs text-secondary-100">
                        Model: {filteredReplacedItems[index].model}
                        <br/>
                        {formatTimestamp(filteredReplacedItems[index].timestamp)}
                      </div>
                    </div>
                    <button 
                      type="button" 
                      class="btn btn-sm h-full preset-filled-secondary-500"
                      onclick={() => removeDecompilation(filteredReplacedItems[index].function_name)}
                    >
                      Remove
                    </button>
                  </div>
                </div>
              </div>
            </VirtualList>
          </div>
        </div>
      {/if}
    </section>
  </div>
</div>