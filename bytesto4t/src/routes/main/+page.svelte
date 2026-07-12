<script lang="ts">
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import PanelFunctions from "../../lib/panels/PanelFunctions.svelte";
  import PanelMain from "../../lib/panels/PanelMain.svelte";
  import PanelStrings from "../../lib/panels/PanelStrings.svelte";
</script>

<main class="w-full h-full bg-surface-950 preset-gradient-surface-grain p-1">
  <PaneGroup direction="horizontal">
    <Pane defaultSize={26}>
      <PaneGroup direction="vertical">
        <Pane>
          <PanelFunctions />
        </Pane>
        <PaneResizer class="h-1 bg-transparent hover:bg-primary-500/40 transition-colors"/>
        <Pane>
          <PanelStrings />
        </Pane>
      </PaneGroup>
    </Pane>
    <PaneResizer class="w-1 bg-transparent hover:bg-primary-500/40 transition-colors"/>
    <Pane defaultSize={74}>
      <PanelMain />
    </Pane>
  </PaneGroup>
</main>
