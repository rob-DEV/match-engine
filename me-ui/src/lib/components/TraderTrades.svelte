<script>
    import {nsToDateTime} from "$lib/utils/utils.ts";
    import {closedTrades} from "$lib/stores/state.ts";
</script>

<div class="bg-gray-800 rounded-lg p-4">
    <div class="flex items-center gap-2 mb-3">
        <h3 class="text-lg font-semibold text-white">Your Trades</h3>
    </div>
    <div class="grid grid-cols-3 gap-2 text-xs text-gray-400 pb-2 border-b border-gray-700">
        <div>Price</div>
        <div>Quantity</div>
        <div>Time</div>
    </div>
    {#if $closedTrades.length === 0}
        <div class="text-gray-500 text-sm py-4 text-center">No trades yet</div>
    {:else}
        {#each $closedTrades as trade}
            <div class="grid grid-cols-3 gap-2 text-sm py-1 text-blue-400">
                <div class="font-mono">${trade.exec_px}</div>
                <div>{trade.exec_qty}</div>
                <div class="text-gray-400 text-xs">{nsToDateTime(trade.exec_ns)}</div>
            </div>
        {/each}
    {/if}
</div>
