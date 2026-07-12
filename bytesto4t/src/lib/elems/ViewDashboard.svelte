<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let dashboardInfo: string = "";

  async function getDashboardInfo() {
    const response = await invoke("get_dashboard_info") as string;
    dashboardInfo = response;
  }

  onMount(() => {
    getDashboardInfo();
  });
</script>

<div class="h-full overflow-hidden">
  <section class="h-full overflow-auto rounded-sm border border-surface-700/70 bg-surface-900/80 p-4">
    <pre class="text-sm leading-relaxed">{dashboardInfo}</pre>
  </section>
</div>
