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
  <div class="card p-2 space-y-2 variant-soft-secondary">
    <header class="card-header">
      <h3 class="h3">
        General information
      </h3>
    </header>
    <section class="card p-4 variant-soft-secondary text-left truncate overflow-x-auto w-full h-full">
      <pre class="overflow-x-auto">{dashboardInfo}</pre>
    </section>
  </div>
</div>