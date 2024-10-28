<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { CodeBlock } from '@skeletonlabs/skeleton';
  import { save } from "@tauri-apps/plugin-dialog";
  import VirtualList from 'svelte-tiny-virtual-list';

  let currentName = "";
  let currentType = "Choose item from left panel";

  async function bytecodeItemSelectedHandler(e: Event) {
    try {
      const ev = e as CustomEvent<{name: string, type: string}>;
      currentName = ev.detail.name;
      currentType = "Type: " + ev.detail.type;
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
      <div class="flex flex-row items-start justify-between">
        <h3 class="h3">
          {currentType}
        </h3>
      </div>
    </header>
    <section class="card p-4 variant-soft-secondary text-left truncate overflow-x-auto w-full h-[calc(100%-3rem)]">
      <pre class="overflow-x-auto">{currentName}</pre>
    </section>
  </div>
</div>