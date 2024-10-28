<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Accordion, AccordionItem } from "@skeletonlabs/skeleton";
  import { Pane, Splitpanes } from "svelte-splitpanes";
  import PanelFunctions from "../../lib/PanelFunctions.svelte";
  import PanelMain from "../../lib/PanelMain.svelte";
  import PanelStrings from "../../lib/PanelStrings.svelte";
  import { onMount } from "svelte";
  
  async function getColorscheme() {
    try {
      const response = await invoke("get_config_colorscheme") as string;
      document.body.setAttribute("data-theme", response);
    } catch (error) {
      console.error("Error getting colorscheme:", error);
    }
  }

  async function getTheme() {
    try {
      const response = await invoke("get_config_theme") as string;
      if (response === "dark") {
        document.documentElement.classList.add("dark");
        document.documentElement.classList.remove("light");
      } else {
        document.documentElement.classList.add("light");
        document.documentElement.classList.remove("dark");
      }
    } catch (error) {
      console.error("Error getting theme:", error);
    }
  }

  onMount(async () => {
    await getColorscheme();
    await getTheme();
  });
</script>

<style>
  :global(.splitpanes__splitter) {
    @apply variant-glass-surface;
    position: relative;
  }
  :global(.splitpanes--vertical > .splitpanes__splitter) {
    width: 4px;
    cursor: col-resize;
  }
  :global(.splitpanes--horizontal > .splitpanes__splitter) {
    height: 4px;
    cursor: row-resize;
  }
  :global(.splitpanes__splitter:hover) {
    @apply variant-glass-secondary;
    transition: 0.6s;
  }
</style>

<div class="flex flex-col h-screen">
  <Splitpanes theme="" horizontal={false} id="splitpanes" class="h-full">
    <Pane size={30} class="h-full">
      <Splitpanes theme="" horizontal={true} class="h-full">
        <Pane size={60} class="h-full">
          <div class="variant-filled-surface-900 h-full">
            <PanelFunctions />
          </div>
        </Pane>
        <Pane class="h-full">
          <div class="variant-filled-surface-900 h-full">
            <PanelStrings />
          </div>
        </Pane>
      </Splitpanes>
    </Pane>
    <Pane>
      <div class="variant-filled-surface-900 h-full">
        <PanelMain />
      </div>
    </Pane>
  </Splitpanes>
</div>
