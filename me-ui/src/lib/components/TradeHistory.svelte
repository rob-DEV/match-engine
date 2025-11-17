<script>
    export let trades = [];
</script>

<div class="bg-gray-800 rounded-lg p-4">
    <h3 class="text-lg font-semibold text-white mb-3">Recent Trades</h3>
    <div class="grid grid-cols-3 gap-2 text-xs text-gray-400 pb-2 border-b border-gray-700">
        <div>Price</div>
        <div>Quantity</div>
        <div>Time</div>
    </div>
    {#if trades.length === 0}
        <div class="text-gray-500 text-sm py-4 text-center">No trades yet</div>
    {:else}
        {#each trades as trade (trade.id)}
            <div class="grid grid-cols-3 gap-2 text-sm py-1 text-blue-400">
                <div class="font-mono">${trade.price.toFixed(2)}</div>
                <div>{trade.quantity}</div>
                <div class="text-gray-400 text-xs">{trade.timestamp}</div>
            </div>
        {/each}
    {/if}
</div>
