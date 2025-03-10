<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { CodeBlock } from '@skeletonlabs/skeleton';
  import hljs from 'highlight.js/lib/core';
  import haxe from 'highlight.js/lib/languages/haxe';
  import 'highlight.js/styles/github-dark.css';
  import { storeHighlightJs } from '@skeletonlabs/skeleton';
  import { writable } from 'svelte/store';
  import type { AIDecompilation, BytecodeItemSelectedEvent } from './types';
  import VirtualList from 'svelte-tiny-virtual-list';
  import { confirm } from '@tauri-apps/plugin-dialog';

  storeHighlightJs.set(hljs);
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

  async function bytecodeItemSelectedHandler(e: Event) {
    try {
      const ev = e as CustomEvent<{name: string, type: string}>;
      
      if (ev.detail.type === "function") {
        currentFunction = ev.detail.name;
        disassemblyCode = await invoke("get_disassembler_info") as string;
      }
    } catch (error) {
      console.error("Error fetching disassembled info:", error);
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

  $effect(() => {
    loadReplacedItems();
  });

  $effect(() => {
    invoke("update_replaced_decompilations")
      .then(() => loadReplacedItems())
      .catch(error => console.error("Failed to update decompilations:", error));
  });

  onMount(() => {
    loadConfig();
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    
    return () => {
      window.removeEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    };
  });

  const filteredReplacedItems = $derived(replacedItems.filter(item => 
    item.function_name.toLowerCase().includes(replacedSearch.toLowerCase())
  ));
</script>

<div class="h-full overflow-y-auto">
  <div class="card p-2 space-y-2 variant-soft-secondary">
    <section class="card p-4 variant-soft-secondary space-y-2">
      <div class="flex justify-between items-center">
        <h4 class="h4">AI Decompiler</h4>
        <button 
          type="button" 
          class="btn variant-soft-secondary"
          onclick={() => showSettings = !showSettings}
        >
          {showSettings ? 'Hide Settings' : 'Show Settings'}
        </button>
      </div>

      {#if showSettings}
        <div class="space-y-2">
          <input 
            type="password" 
            class="input variant-form-material w-full" 
            placeholder="OpenRouter API Key"
            bind:value={openrouterKey}
            onchange={saveConfig}
          />
          <select 
            class="select variant-form-material w-full" 
            bind:value={selectedModel}
            onchange={saveConfig}
          >
            {#each models as model}
              <option value={model}>{model}</option>
            {/each}
          </select>
          <div class="space-y-1">
            <label class="text-sm">Custom Prompt</label>
            <textarea
              class="textarea variant-form-material w-full"
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
          class="input variant-form-material flex-1" 
          placeholder="No function selected"
          value={currentFunction}
          disabled
        />
        <button 
          type="button" 
          class="btn variant-filled-primary" 
          onclick={decompileCode}
          disabled={isLoading || !currentFunction}
        >
          {isLoading ? 'Decompiling...' : 'Decompile'}
        </button>
      </div>
    </section>

    {#if decompilationResult}
      <section class="card p-4 variant-soft-secondary">
        <div class="flex justify-between items-center mb-2">
          <div class="text-sm">Result length: {decompilationResult.length}</div>
          <div class="flex gap-2">
            <button 
              type="button" 
              class="btn variant-filled-secondary"
              onclick={saveDecompilation}
            >
              Replace Original
            </button>
          </div>
        </div>
        <textarea
          class="textarea variant-form-material w-full font-mono text-sm mb-2"
          rows="20"
          bind:value={decompilationResult}
        />
        <CodeBlock 
          language="haxe" 
          code={decompilationResult} 
          background="bg-secondary-900"
        />
      </section>
    {/if}

    <section class="card p-4 variant-soft-secondary space-y-2">
      <div class="flex justify-between items-center">
        <h4 class="h4">Replaced Functions</h4>
        <div class="flex gap-2">
          <button 
            type="button" 
            class="btn variant-soft-secondary"
            onclick={() => showReplaced = !showReplaced}
          >
            {showReplaced ? 'Hide Replaced' : 'Show Replaced'}
          </button>
          {#if filteredReplacedItems.length > 0}
            <button 
              type="button" 
              class="btn variant-filled-error"
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
            class="input variant-form-material w-full" 
            placeholder="Search replaced functions..."
            bind:value={replacedSearch}
          />

          <div class="h-48 overflow-y-auto">
            <VirtualList 
              width="100%" 
              height="100%" 
              itemCount={filteredReplacedItems.length} 
              itemSize={80}
              overscanCount={5}
            >
              <div slot="item" let:index let:style {style}>
                <div class="card p-2 variant-glass-primary mb-1">
                  <div class="flex justify-between items-center">
                    <div class="flex-1">
                      <div class="font-semibold truncate">
                        {filteredReplacedItems[index].function_name}
                      </div>
                      <div class="text-xs text-secondary-400">
                        Model: {filteredReplacedItems[index].model}
                        <br/>
                        {formatTimestamp(filteredReplacedItems[index].timestamp)}
                      </div>
                    </div>
                    <button 
                      type="button" 
                      class="btn btn-sm variant-filled-error"
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