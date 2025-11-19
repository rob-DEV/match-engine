<script>
    import {market} from '$lib/stores/market';
    import LastPrice from '$lib/components/LastPrice.svelte';
    import OrderBook from '$lib/components/OrderBook.svelte';
    import TradeHistory from '$lib/components/TradeHistory.svelte';

    let showFlash = false;
</script>

<svelte:head>
    <title>Matching Engine</title>
</svelte:head>

<div class="min-h-screen bg-gray-900 p-6">
    <div class="max-w-7xl mx-auto">
        <!-- Header -->
        <div class="text-center mb-8">
            <h1 class="text-4xl font-bold text-white mb-2">Matching Engine</h1>
            <p class="text-gray-400">Real-time order book and trade execution</p>
        </div>

        <!-- Last Price Display -->
        <LastPrice lastPrice={$market.last_px} {showFlash}/>


        <!-- Order Entry -->
        <!--        <TradeInput on:addOrder={addOrder}/>-->

        <!-- Order Books -->
        <div class="grid md:grid-cols-2 gap-6 mb-6">
            <OrderBook type="buy" orders={$market.l2.bids}/>
            <OrderBook type="sell" orders={$market.l2.asks}/>
        </div>

        <TradeHistory trades={$market.trades}/>
    </div>
</div>