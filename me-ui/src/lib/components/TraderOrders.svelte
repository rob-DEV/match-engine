<script lang="ts">
    import {wsMessages} from '$lib/stores/event_stream.ts';
    import {derived} from 'svelte/store';

    // Filter only order/trade messages for this component
    export const orders = derived(wsMessages, ($wsMessages) =>
        $wsMessages.filter((m) => m.type === 'OrderAck')
    );
</script>

<div class="bg-gray-800 rounded-lg p-4">
    <div class="flex items-center gap-2 mb-3">
        <h3 class="text-lg font-semibold text-white">Your Orders</h3>
    </div>
    <div class="grid grid-cols-3 gap-2 text-xs text-gray-400 pb-2 border-b border-gray-700">
        <div>Price</div>
        <div>Quantity</div>
        <div>Status</div>
    </div>
    {#if $orders.length === 0}
        <div class="text-gray-500 text-sm py-4 text-center">No orders</div>
    {:else}
        {#each $orders as order}
            {#if order.px > 0}
                <div class="grid grid-cols-3 gap-2 text-sm py-1 text-blue-400">
                    <div class="font-mono">${order.px}</div>
                    <div>{order.qty}</div>
                </div>
            {/if}
        {/each}
    {/if}
</div>
