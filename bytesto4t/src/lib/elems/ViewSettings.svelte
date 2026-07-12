<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  
  let selectedTheme = document.body.getAttribute("data-theme") || "bytesto4t";
  
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

  onMount(async () => {
    try {
      const theme = await invoke("get_config_colorscheme") as string;
      selectedTheme = theme || "bytesto4t";
      document.body.setAttribute("data-theme", selectedTheme);
    } catch (error) {
      console.error("Error getting colorscheme:", error);
    }
  });
</script>

<div class="h-full overflow-y-auto">
  <section class="max-w-xl rounded-sm border border-surface-700/70 preset-gradient-surface-grain-medium bg-surface-900/80 p-4 space-y-2">
      <label class="block text-sm font-semibold text-surface-100" for="theme-select">Theme</label>
      <select id="theme-select" class="select bg-surface-800 focus:outline-none" size=1 bind:value={selectedTheme} on:change={changeColorscheme}>
        <option value="bytesto4t">Bytesto4t</option>
        <option value="Firow">Firow</option>
        <option value="catppuccin">Catppuccin</option>
        <option value="cerberus">Cerberus</option>
        <option value="concord">Concord</option>
        <option value="crimson">Crimson</option>
        <option value="fennec">Fennec</option>
        <option value="hamlindigo">Hamlindigo</option>
        <option value="legacy">Legacy</option>
        <option value="mint">Mint</option>
        <option value="modern">Modern</option>
        <option value="mona">Mona</option>
        <option value="nosh">Nosh</option>
        <option value="nouveau">Nouveau</option>
        <option value="pine">Pine</option>
        <option value="reign">Reign</option>
        <option value="rocket">Rocket</option>
        <option value="rose">Rose</option>
        <option value="sahara">Sahara</option>
        <option value="seafoam">Seafoam</option>
        <option value="terminus">Terminus</option>
        <option value="vintage">Vintage</option>
        <option value="vox">Vox</option>
        <option value="wintry">Wintry</option>
      </select>
  </section>
</div>
