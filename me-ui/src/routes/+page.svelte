<script>
    import LastPrice from '$lib/components/LastPrice.svelte';
    import TradeInput from '$lib/components/TradeInput.svelte';
    import OrderBook from '$lib/components/OrderBook.svelte';
    import TradeHistory from '$lib/components/TradeHistory.svelte';

    let buyOrders = [];
    let sellOrders = [];
    let trades = [];
    let lastPrice = 100.00;
    let showFlash = false;

    function addOrder({detail}) {
        const {orderType, price, quantity} = detail;
        const p = parseFloat(price);
        const q = parseInt(quantity);
        if (!p || !q) return;
        const order = {id: Date.now(), price: p, quantity: q, timestamp: new Date().toLocaleTimeString()};
        if (orderType === 'buy') buyOrders = [...buyOrders, order];
        else sellOrders = [...sellOrders, order];

        setTimeout(matchOrders, 10);
    }

    function matchOrders() {
        // Copy arrays and sort
        const buys = [...buyOrders].sort((a, b) => b.price - a.price);
        const sells = [...sellOrders].sort((a, b) => a.price - b.price);

        const newTrades = [];
        const remainingBuys = [];
        const remainingSells = [];

        let i = 0, j = 0;

        while (i < buys.length && j < sells.length) {
            const buy = buys[i];
            const sell = sells[j];

            // Match if buy price >= sell price
            if (buy.price >= sell.price) {
                const matchedQty = Math.min(buy.quantity, sell.quantity);
                const matchedPrice = sell.price;

                // Record trade
                newTrades.push({
                    id: Date.now() + Math.random(), // unique id
                    price: matchedPrice,
                    quantity: matchedQty,
                    timestamp: new Date().toLocaleTimeString()
                });

                // Update last traded price
                lastPrice = matchedPrice;

                // Flash animation trigger
                showFlash = true;
                setTimeout(() => showFlash = false, 1000);

                // Decrease quantities
                buy.quantity -= matchedQty;
                sell.quantity -= matchedQty;

                if (buy.quantity === 0) i++;
                if (sell.quantity === 0) j++;
            } else {
                // No more matches possible
                break;
            }
        }

        // Collect remaining unmatched orders
        for (let k = i; k < buys.length; k++) {
            if (buys[k].quantity > 0) remainingBuys.push(buys[k]);
        }
        for (let k = j; k < sells.length; k++) {
            if (sells[k].quantity > 0) remainingSells.push(sells[k]);
        }

        // Update state if there were trades
        if (newTrades.length > 0) {
            trades = [...newTrades, ...trades].slice(0, 20); // keep last 20 trades
            buyOrders = remainingBuys;
            sellOrders = remainingSells;
        }
    }

    $: sortedBuyOrders = [...buyOrders].sort((a, b) => b.price - a.price);
    $: sortedSellOrders = [...sellOrders].sort((a, b) => a.price - b.price);
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
        <LastPrice {lastPrice} {showFlash}/>


        <!-- Order Entry -->
        <TradeInput on:addOrder={addOrder}/>

        <!-- Order Books -->
        <div class="grid md:grid-cols-2 gap-6 mb-6">
            <OrderBook type="buy" orders={sortedBuyOrders}/>
            <OrderBook type="sell" orders={sortedSellOrders}/>
        </div>

        <TradeHistory {trades}/>
    </div>
</div>