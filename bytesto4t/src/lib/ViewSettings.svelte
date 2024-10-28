<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  
  let selectedTheme = document.body.getAttribute("data-theme") || "skeleton";
  let selectedThemeMode = document.documentElement.classList.contains("dark") ? "dark" : "light";
  
  
  async function changeColorscheme(e: Event) {
    const target = e.target as HTMLSelectElement;
    document.body.setAttribute("data-theme", target.value);

    try {
      await invoke("set_config_colorscheme", { colorscheme: target.value });
      await invoke("save_config");
    } catch (error) {
      console.error("Error setting colorscheme:", error);
    }
  }

  async function changeTheme(e: Event) {
    const target = e.target as HTMLSelectElement;
    if (target.value === "dark") {
      document.documentElement.classList.add("dark");
      document.documentElement.classList.remove("light");
    } else {
      document.documentElement.classList.add("light");
      document.documentElement.classList.remove("dark");
    }
    selectedThemeMode = target.value;

    try {
      await invoke("set_config_theme", { theme: target.value });
      await invoke("save_config");
    } catch (error) {
      console.error("Error setting theme:", error);
    }
  }
  onMount(() => {
    if (selectedThemeMode === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.add("light");
    }
  });
</script>

<div class="h-full overflow-y-auto">
  <div class="card p-2 space-y-2 variant-soft-secondary">
    <section class="card p-4 variant-soft-secondary space-y-2">
      <h4 class="h4">Theme</h4>
      <select class="select variant-form-material variant-soft-secondary" bind:value={selectedTheme} on:change={changeColorscheme}>
        <option value="skeleton">Skeleton</option>
        <option value="wintry">Wintry</option>
        <option value="modern">Modern</option>
        <option value="rocket">Rocket</option>
        <option value="seafoam">Seafoam</option>
        <option value="vintage">Vintage</option>
        <option value="sahara">Sahara</option>
        <option value="hamlindigo">Hamlindigo</option>
        <option value="gold-nouveau">Gold Nouveau</option>
        <option value="crimson">Crimson</option>
      </select>
      <select class="select variant-form-material variant-soft-secondary" bind:value={selectedThemeMode} on:change={changeTheme}>
        <option value="dark">Dark</option>
        <option value="light">Light</option>
      </select>
    </section>
  </div>
</div>