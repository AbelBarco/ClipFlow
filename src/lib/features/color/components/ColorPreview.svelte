<script lang="ts">
  interface Props {
    color: string;
    size?: 'sm' | 'md' | 'lg';
  }

  let { color, size = 'md' }: Props = $props();

  const sizes = {
    sm: 'w-6 h-6',
    md: 'w-8 h-8',
    lg: 'w-12 h-12'
  };
</script>

<div
  class="{sizes[size]} rounded-lg border border-surface-300 dark:border-surface-600 overflow-hidden relative"
  style="background-color: {color}"
  title={color}
  role="img"
  aria-label={`Color preview: ${color}`}
>
  <div class="absolute inset-0 bg-[conic-gradient(from_90deg_at_50%_50%,transparent_50%,white_50%),conic-gradient(from_-90deg_at_50%_50%,transparent_50%,black_50%)] bg-[length:8px_8px]"></div>
</div>