<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { CodeBlock } from '@skeletonlabs/skeleton';
  import { save } from "@tauri-apps/plugin-dialog";
  import VirtualList from 'svelte-tiny-virtual-list';

  let inspectorContent = "";
  let inspectorTitle = "Inspector";

  async function bytecodeItemSelectedHandler(e: Event) {
    try {
      const ev = e as CustomEvent<{name: string, type: string}>;

      const index = ev.detail.name.split('@').pop();
      if (!index) {
        throw new Error("Invalid item name format");
      }

      updateInspector();
    } catch (error) {
      console.log("Error fetching inspector info:", error);
    }
  }

  async function updateInspector() {
    const inspectorInfo = await invoke("get_inspector_info") as string;
    if (inspectorInfo) {
      inspectorContent = inspectorInfo;
    }
  }

  onMount(() => {
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    updateInspector();
  });
</script>

<div class="h-full overflow-y-auto">
  <div class="card p-2 space-y-2 variant-soft-secondary h-full">
    <header class="card-header h-12">
      <div class="flex flex-row items-start justify-between">
        <h3 class="h3 truncate">
          {inspectorTitle}
        </h3>
      </div>
    </header>
    <section class="card p-4 variant-soft-secondary text-left w-full h-[calc(100%-3rem)]">
      <pre class="overflow-auto h-full whitespace-pre-wrap">{inspectorContent}</pre>
    </section>
  </div>
</div>