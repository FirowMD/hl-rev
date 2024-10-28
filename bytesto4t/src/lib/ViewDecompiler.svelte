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

  let functionName = "Choose function to decompile";
  let decompiledCode = "";

  async function bytecodeItemSelectedHandler(e: Event) {
    try {
      const ev = e as CustomEvent<{name: string, type: string}>;
      if (ev.detail.type == "function") {
        functionSelectedHandler(e);
      } else if (ev.detail.type == "class") {
        typeSelectedHandler(e);
      }
    } catch (error) {
      console.log("Error fetching decompiled code:", error);
    }
  }

  async function functionSelectedHandler(e: Event) {
    try {
      const ev = e as CustomEvent<{name: string, type: string}>;
      functionName = ev.detail.name;
      var findex = functionName.split("@")[1];
      findex = "@" + findex;
      const response = await invoke("get_decompiled_code", {functionIndex: findex}) as string;
      if (response !== null) {
        decompiledCode = response;
      }
    } catch (error) {
      console.log("Error fetching decompiled code:", error);
    }
  }

  async function typeSelectedHandler(e: Event) {
    try {
      decompiledCode = ""
      const ev = e as CustomEvent<{name: string, type: string}>;
      functionName = ev.detail.name;
      var findex = functionName.split("@")[1];
      findex = "@" + findex;
      const response = await invoke("get_decompiled_type", {typeIndex: findex}) as string;
      if (response !== null) {
        console.log(response);
        decompiledCode = response;
      }
    } catch (error) {
      console.log("Error fetching decompiled code:", error);
    }
  }

  onMount(() => {
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
  });

</script>
  
  
<div class="h-full overflow-y-auto">
  <div class="card p-2 space-y-2 variant-soft-secondary h-full">
    <header class="card-header h-12">
      <h3 class="h3">
        {functionName}
      </h3>
    </header>
    <section class="p-2 h-[calc(100%-3rem)]">
      <CodeBlock class="h-full overflow-y-auto" language="haxe" background="bg-secondary-900" lineNumbers={true} code={decompiledCode} />
    </section>
  </div>
</div>
