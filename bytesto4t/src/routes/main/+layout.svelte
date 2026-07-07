<script lang="ts">
  let { children } = $props();
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import "../../app.css";

  async function getColorscheme() {
    try {
      const response = await invoke("get_config_colorscheme") as string;
      document.body.setAttribute("data-theme", response);
    } catch (error) {
      console.error("Error getting colorscheme:", error);
    }
  }

  onMount(async () => {
    await getColorscheme();
  });
</script>
{@render children()}