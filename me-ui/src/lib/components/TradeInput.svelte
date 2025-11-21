<script>
    import {sendOrder} from "../stores/event_stream.ts";
    import {boundedRandomNumber} from "$lib/utils/utils.ts";

    export let side = 'buy';
    export let px = '';
    export let qty = '';

    const CLIENT_TRADER_ID = boundedRandomNumber(100_000, 10_000_000);

    function placeOrder() {
        let order = {
            type: 'NewOrder',
            data: {side, px, qty}
        };
        sendOrder(order);

        px = '';
        qty = '';
        console.log("Sent order:", order);
    }
</script>

<div class="bg-gray-800 rounded-lg p-6 mb-6">
    <h2 class="text-xl font-semibold text-white mb-4">Place Order</h2>
    <div class="space-y-4">
        <div class="flex gap-4">
            <button on:click={() => side = 'buy'}
                    class="flex-1 py-3 rounded-lg font-semibold transition {side ==='buy'? 'bg-green-600 text-white':'bg-gray-700 text-gray-300 hover:bg-gray-600'}">
                Buy
            </button>
            <button on:click={() => side = 'sell'}
                    class="flex-1 py-3 rounded-lg font-semibold transition {side ==='sell'? 'bg-red-600 text-white':'bg-gray-700 text-gray-300 hover:bg-gray-600'}">
                Sell
            </button>
        </div>
        <div class="grid grid-cols-2 gap-4">
            <input type="number" step="0.01" placeholder="Price" bind:value={px} class="w-full bg-gray-700 text-white rounded-lg px-4 py-2"/>
            <input type="number" placeholder="Quantity" bind:value={qty} class="w-full bg-gray-700 text-white rounded-lg px-4 py-2"/>
        </div>
        <button on:click={placeOrder} class="{side==='buy'? 'bg-green-600':'bg-red-600'} w-full py-3 rounded-lg text-white">
            Place {side} </button>
    </div>
</div>
