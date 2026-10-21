<script lang="ts">
  import { label } from '../format';
  let { value = 'unknown' }: { value?: string } = $props();
</script>

<span
  class:attention={[
    'partial',
    'requiresPermission',
    'temporarilyUnavailable',
    'failed',
    'interrupted',
    'unverified',
    'skipped',
    'attention',
  ].includes(value)}
  class:positive={value === 'supported' || value === 'complete'}
  class="status"><span class="status-dot"></span>{label(value)}</span
>
