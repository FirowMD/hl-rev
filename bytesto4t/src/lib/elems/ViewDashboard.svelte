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

<div class="h-full">
  <section class="p-4 preset-outlined-surface-500 bg-surface-900 text-left overflow-y-auto w-full h-fit">
    <pre class="h-fit">{dashboardInfo}</pre>
  </section>
</div>