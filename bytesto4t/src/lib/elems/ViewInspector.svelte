<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
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
  <div class="p-2 space-y-2 h-full">
    <header class="flex items-center justify-between p-3 h-12">
      <h5 class="h5">Inspector</h5>
    </header>
    <section class="card preset-outlined-surface-500 bg-surface-900 p-4">
      <pre>{inspectorContent}</pre>
    </section>
  </div>
</div>