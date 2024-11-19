<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { CodeBlock } from '@skeletonlabs/skeleton';
  import hljs from 'highlight.js/lib/core';
  import haxe from 'highlight.js/lib/languages/haxe';
  import 'highlight.js/styles/github-dark.css';
  import { storeHighlightJs } from '@skeletonlabs/skeleton';

  storeHighlightJs.set(hljs);
  hljs.registerLanguage('haxe', haxe);

  let decompilerTitle = "Decompiler";
  let functionName = "";
  let decompiledCode = "";

  async function bytecodeItemSelectedHandler(e: Event) {
    try {
      const ev = e as CustomEvent<{name: string, type: string}>;
      const itemType = ev.detail.type;

      if (itemType === "function" || itemType === "class") {
        functionName = ev.detail.name;
        updateDecompiler();
      }
    } catch (error) {
      console.log("Error fetching decompiled info:", error);
    }
  }

  async function updateDecompiler() {
    const response = await invoke("get_decompiled_info") as string;
    if (response !== null) {
      decompiledCode = response;
    }
  }

  onMount(() => {
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    updateDecompiler();
  });
</script>

<div class="h-full overflow-y-auto">
  <div class="card p-2 space-y-2 variant-soft-secondary h-full">
    <header class="card-header h-12">
      <div class="flex flex-row items-start justify-between">
        <h3 class="h3">
          {decompilerTitle}
        </h3>
        {#if functionName !== ""}
          <h3 class="h3">
            {functionName}
          </h3>
        {/if}
      </div>
    </header>
    <section class="p-2 h-[calc(100%-3rem)]">
      <CodeBlock class="h-full overflow-y-auto" language="haxe" background="bg-secondary-900" lineNumbers={true} code={decompiledCode} />
    </section>
  </div>
</div>
