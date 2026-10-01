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
  title={color}
  role="img"
  aria-label={color}
>
  <!-- Damero debajo: solo se ve a través de colores con transparencia. -->
  <div
    class="absolute inset-0 bg-[conic-gradient(from_90deg_at_50%_50%,#d4d4d8_25%,#fafafa_0_50%,#d4d4d8_0_75%,#fafafa_0)] bg-[length:8px_8px] dark:bg-[conic-gradient(from_90deg_at_50%_50%,#3f3f46_25%,#27272a_0_50%,#3f3f46_0_75%,#27272a_0)]"
    aria-hidden="true"
  ></div>
  <div class="absolute inset-0" style="background-color: {color}" aria-hidden="true"></div>
</div>