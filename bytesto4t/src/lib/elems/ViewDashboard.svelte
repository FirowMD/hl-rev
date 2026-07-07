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

<div class="h-full overflow-y-auto">
  <div class="p-2 space-y-2 h-full">
    <header class="flex items-center justify-between p-3 h-12">
      <h5 class="h5">Dashboard</h5>
    </header>
    <section class="card preset-outlined-surface-500 bg-surface-900 p-4">
      <pre>{dashboardInfo}</pre>
    </section>
  </div>
</div>