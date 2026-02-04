<!--
  ReferencePeek — on hover shows artifact summary.
  No content load. Click opens explicit rehydration view.
  Per docs/11-menubar-ui-spec.md.
-->
<script lang="ts">
  // Reference peek is passive — only activates on hover over reference handles.
  // MVP: placeholder for future hover-based artifact preview.
</script>

<!-- Invisible until reference handles are hovered in another component -->
<div class="reference-peek" aria-hidden="true"></div>

<style>
  .reference-peek {
    display: none;
  }
</style>
