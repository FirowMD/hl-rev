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

  // Create a store for replaced functions
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
      console.log("Bytecode item selected:", ev.detail);
      
      if (ev.detail.type === "function") {
        currentFunction = ev.detail.name;
        disassemblyCode = await invoke("get_disassembler_info") as string;
        console.log("Loaded disassembly for function:", currentFunction);
        console.log("Disassembly length:", disassemblyCode.length);
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

    console.log("Current function:", currentFunction);
    console.log("Disassembly length:", disassemblyCode.length);
    console.log("Disassembly sample:", disassemblyCode.substring(0, 200));

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

      console.log("Request body:", JSON.stringify(requestBody, null, 2));

      const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${openrouterKey}`,
          'Content-Type': 'application/json',
          'HTTP-Referer': window.location.origin,
          'X-Title': 'BytesTo4T',
        },
        body: JSON.stringify(requestBody),
      });

      console.log("Response status:", response.status);
      const data = await response.json();
      console.log("Response data:", data);

      if (!data.choices?.[0]?.message?.content) {
        throw new Error("Invalid response format: " + JSON.stringify(data));
      }

      decompilationResult = data.choices[0].message.content;
      console.log("Decompilation result set to:", decompilationResult);
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
      
      // Update replaced functions list
      replacedFunctions.update(set => {
        set.add(currentFunction);
        return set;
      });

      // Dispatch event to update decompiler view
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
      
      // Update replaced functions list
      replacedFunctions.update(set => {
        set.delete(functionName);
        return set;
      });

      // Notify decompiler to refresh if currently showing this function
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

  // Update onMount to load replaced items
  onMount(() => {
    // Execute async operations but don't return their promise
    loadConfig();
    loadReplacedItems();
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    
    return () => {
      window.removeEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    };
  });

  // Filter replaced items based on search
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
          on:click={() => showSettings = !showSettings}
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
            on:change={saveConfig}
          />
          <select 
            class="select variant-form-material w-full" 
            bind:value={selectedModel}
            on:change={saveConfig}
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
              on:change={saveConfig}
            />
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
          on:click={decompileCode}
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
              on:click={saveDecompilation}
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
        <button 
          type="button" 
          class="btn variant-soft-secondary"
          on:click={() => showReplaced = !showReplaced}
        >
          {showReplaced ? 'Hide Replaced' : 'Show Replaced'}
        </button>
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
                      on:click={() => removeDecompilation(filteredReplacedItems[index].function_name)}
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