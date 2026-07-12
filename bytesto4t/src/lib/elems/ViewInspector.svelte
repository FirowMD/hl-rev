<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let inspectorContent = "";

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

<div class="h-full overflow-hidden">
  <section class="h-full overflow-auto rounded-sm border border-surface-700/70 bg-surface-900/80 p-4">
    <pre class="text-sm leading-relaxed">{inspectorContent}</pre>
  </section>
</div>
