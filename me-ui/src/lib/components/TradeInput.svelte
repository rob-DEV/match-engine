<script>
    import {createEventDispatcher} from 'svelte';

    const dispatch = createEventDispatcher();

    export let orderType = 'buy';
    export let price = '';
    export let quantity = '';

    function addOrder() {
        dispatch('addOrder', {orderType, price, quantity});
    }
</script>

<div class="bg-gray-800 rounded-lg p-6 mb-6">
    <h2 class="text-xl font-semibold text-white mb-4">Place Order</h2>
    <div class="space-y-4">
        <div class="flex gap-4">
            <button on:click={() => orderType = 'buy'}
                    class="flex-1 py-3 rounded-lg font-semibold transition {orderType==='buy'? 'bg-green-600 text-white':'bg-gray-700 text-gray-300 hover:bg-gray-600'}">
                Buy
            </button>
            <button on:click={() => orderType = 'sell'}
                    class="flex-1 py-3 rounded-lg font-semibold transition {orderType==='sell'? 'bg-red-600 text-white':'bg-gray-700 text-gray-300 hover:bg-gray-600'}">
                Sell
            </button>
        </div>
        <div class="grid grid-cols-2 gap-4">
            <input type="number" step="0.01" placeholder="Price" bind:value={price} class="w-full bg-gray-700 text-white rounded-lg px-4 py-2"/>
            <input type="number" placeholder="Quantity" bind:value={quantity} class="w-full bg-gray-700 text-white rounded-lg px-4 py-2"/>
        </div>
        <!--        <button on:click={addOrder} class="{orderType==='buy'? 'bg-green-600':'bg-red-600'} w-full py-3 rounded-lg text-white">-->
        <button class="{orderType==='buy'? 'bg-green-600':'bg-red-600'} w-full py-3 rounded-lg text-white">
            Place {orderType}</button>
    </div>
</div>
