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

<div class="h-full">
  <section class="p-4 preset-outlined-surface-500 bg-surface-900 text-left overflow-y-auto w-full h-full">
    <pre class="h-full">{inspectorContent}</pre>
  </section>
</div>