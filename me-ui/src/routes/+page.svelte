<script>
    let buyOrders = [];
    let sellOrders = [];
    let trades = [];
    let orderType = 'buy';
    let price = '';
    let quantity = '';
    let lastPrice = 100.00;
    let showFlash = false;

    function matchOrders() {
        const buys = [...buyOrders].sort((a, b) => b.price - a.price);
        const sells = [...sellOrders].sort((a, b) => a.price - b.price);

        const newTrades = [];
        const remainingBuys = [];
        const remainingSells = [];

        let i = 0, j = 0;

        while (i < buys.length && j < sells.length) {
            const buy = buys[i];
            const sell = sells[j];

            if (buy.price >= sell.price) {
                const matchedQty = Math.min(buy.quantity, sell.quantity);
                const matchedPrice = sell.price;

                newTrades.push({
                    id: Date.now() + Math.random(),
                    price: matchedPrice,
                    quantity: matchedQty,
                    timestamp: new Date().toLocaleTimeString()
                });

                lastPrice = matchedPrice;
                showFlash = true;
                setTimeout(() => {
                    showFlash = false;
                }, 1000);

                buy.quantity -= matchedQty;
                sell.quantity -= matchedQty;

                if (buy.quantity === 0) i++;
                if (sell.quantity === 0) j++;
            } else {
                break;
            }
        }

        for (let k = i; k < buys.length; k++) {
            if (buys[k].quantity > 0) remainingBuys.push(buys[k]);
        }
        for (let k = j; k < sells.length; k++) {
            if (sells[k].quantity > 0) remainingSells.push(sells[k]);
        }

        if (newTrades.length > 0) {
            trades = [...newTrades, ...trades].slice(0, 20);
            buyOrders = remainingBuys;
            sellOrders = remainingSells;
        }
    }

    function addOrder() {
        const p = parseFloat(price);
        const q = parseInt(quantity);

        if (!p || !q || p <= 0 || q <= 0) return;

        const order = {
            id: Date.now(),
            price: p,
            quantity: q,
            timestamp: new Date().toLocaleTimeString()
        };

        if (orderType === 'buy') {
            buyOrders = [...buyOrders, order];
        } else {
            sellOrders = [...sellOrders, order];
        }

        price = '';
        quantity = '';

        setTimeout(() => matchOrders(), 10);
    }

    function handleKeyPress(e) {
        if (e.key === 'Enter') {
            addOrder();
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
        <div class="bg-gradient-to-r from-blue-600 to-purple-600 rounded-lg p-6 mb-6 relative overflow-hidden">
            {#if showFlash}
                <div class="absolute inset-0 bg-white opacity-20 animate-pulse"></div>
            {/if}
            <div class="relative z-10">
                <div class="text-gray-200 text-sm mb-1">Last Trade Price</div>
                <div class="text-5xl font-bold text-white">${lastPrice.toFixed(2)}</div>
                {#if showFlash}
                    <div class="text-green-300 text-sm mt-2 animate-pulse">
                        ✓ Trade executed!
                    </div>
                {/if}
            </div>
        </div>

        <!-- Order Entry -->
        <div class="bg-gray-800 rounded-lg p-6 mb-6">
            <h2 class="text-xl font-semibold text-white mb-4 flex items-center gap-2">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
                Place Order
            </h2>
            <div class="space-y-4">
                <div class="flex gap-4">
                    <button
                            on:click={() => orderType = 'buy'}
                            class="flex-1 py-3 rounded-lg font-semibold transition {orderType === 'buy' ? 'bg-green-600 text-white' : 'bg-gray-700 text-gray-300 hover:bg-gray-600'}"
                    >
                        Buy
                    </button>
                    <button
                            on:click={() => orderType = 'sell'}
                            class="flex-1 py-3 rounded-lg font-semibold transition {orderType === 'sell' ? 'bg-red-600 text-white' : 'bg-gray-700 text-gray-300 hover:bg-gray-600'}"
                    >
                        Sell
                    </button>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm text-gray-400 mb-2">Price ($)</label>
                        <input
                                type="number"
                                step="0.01"
                                bind:value={price}
                                on:keypress={handleKeyPress}
                                class="w-full bg-gray-700 text-white rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 outline-none"
                                placeholder="0.00"
                        />
                    </div>
                    <div>
                        <label class="block text-sm text-gray-400 mb-2">Quantity</label>
                        <input
                                type="number"
                                bind:value={quantity}
                                on:keypress={handleKeyPress}
                                class="w-full bg-gray-700 text-white rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 outline-none"
                                placeholder="0"
                        />
                    </div>
                </div>
                <button
                        on:click={addOrder}
                        class="w-full py-3 rounded-lg font-semibold transition {orderType === 'buy' ? 'bg-green-600 hover:bg-green-700 text-white' : 'bg-red-600 hover:bg-red-700 text-white'}"
                >
                    Place {orderType === 'buy' ? 'Buy' : 'Sell'} Order
                </button>
            </div>
        </div>

        <!-- Order Books -->
        <div class="grid md:grid-cols-2 gap-6 mb-6">
            <!-- Buy Orders -->
            <div class="bg-gray-800 rounded-lg p-4">
                <div class="flex items-center gap-2 mb-3">
                    <svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6"/>
                    </svg>
                    <h3 class="text-lg font-semibold text-white">Buy Orders</h3>
                </div>
                <div class="space-y-1">
                    <div class="grid grid-cols-3 gap-2 text-xs text-gray-400 pb-2 border-b border-gray-700">
                        <div>Price</div>
                        <div>Quantity</div>
                        <div>Time</div>
                    </div>
                    {#if sortedBuyOrders.length === 0}
                        <div class="text-gray-500 text-sm py-4 text-center">No orders</div>
                    {:else}
                        {#each sortedBuyOrders as order (order.id)}
                            <div class="grid grid-cols-3 gap-2 text-sm py-1 text-green-400">
                                <div class="font-mono">${order.price.toFixed(2)}</div>
                                <div>{order.quantity}</div>
                                <div class="text-gray-400 text-xs">{order.timestamp}</div>
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>

            <!-- Sell Orders -->
            <div class="bg-gray-800 rounded-lg p-4">
                <div class="flex items-center gap-2 mb-3">
                    <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 17h8m0 0V9m0 8l-8-8-4 4-6-6"/>
                    </svg>
                    <h3 class="text-lg font-semibold text-white">Sell Orders</h3>
                </div>
                <div class="space-y-1">
                    <div class="grid grid-cols-3 gap-2 text-xs text-gray-400 pb-2 border-b border-gray-700">
                        <div>Price</div>
                        <div>Quantity</div>
                        <div>Time</div>
                    </div>
                    {#if sortedSellOrders.length === 0}
                        <div class="text-gray-500 text-sm py-4 text-center">No orders</div>
                    {:else}
                        {#each sortedSellOrders as order (order.id)}
                            <div class="grid grid-cols-3 gap-2 text-sm py-1 text-red-400">
                                <div class="font-mono">${order.price.toFixed(2)}</div>
                                <div>{order.quantity}</div>
                                <div class="text-gray-400 text-xs">{order.timestamp}</div>
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>
        </div>

        <!-- Trade History -->
        <div class="bg-gray-800 rounded-lg p-4">
            <h3 class="text-lg font-semibold text-white mb-3">Recent Trades</h3>
            <div class="space-y-1">
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
        </div>
    </div>
</div>