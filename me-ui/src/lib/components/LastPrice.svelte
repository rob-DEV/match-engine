<script>
    export let lastPrice = 0;
    export let showFlash = false;
</script>

<div class="bg-gradient-to-r from-blue-600 to-purple-600 rounded-lg p-6 mb-6 relative overflow-hidden">
    {#if showFlash}
        <div class="absolute inset-0 bg-white opacity-20 animate-pulse"></div>
    {/if}
    <div class="relative z-10">
        <div class="text-gray-200 text-sm mb-1">Last Trade Price</div>
        <div class="text-5xl font-bold text-white">${lastPrice.toFixed(2)}</div>
        {#if showFlash}
            <div class="text-green-300 text-sm mt-2 animate-pulse">✓ Trade executed!</div>
        {/if}
    </div>
</div>
