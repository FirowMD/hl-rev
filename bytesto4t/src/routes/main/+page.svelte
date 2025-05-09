<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import PanelFunctions from "../../lib/panels/PanelFunctions.svelte";
  import PanelMain from "../../lib/panels/PanelMain.svelte";
  import PanelStrings from "../../lib/panels/PanelStrings.svelte";
</script>

<main class="w-full h-full bg-gradient-to-b from-surface-950 to-surface-950">
  <PaneGroup direction="horizontal">
    <Pane defaultSize={26}>
      <PaneGroup direction="vertical">
        <Pane>
          <PanelFunctions />
        </Pane>
        <PaneResizer class="h-1 bg-surface-800"/>
        <Pane>
          <PanelStrings />
        </Pane>
      </PaneGroup>
    </Pane>
    <PaneResizer class="w-1 bg-surface-800"/>
    <Pane defaultSize={74}>
      <PanelMain />
    </Pane>
  </PaneGroup>
</main>
