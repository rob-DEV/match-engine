import {writable} from 'svelte/store';

export const market = writable({
    l1: {best_bid: 0, best_ask: 0, last_price: 0},
    l2: {
        bids: Array(10).fill({px: 0, qty: 0}),
        asks: Array(10).fill({px: 0, qty: 0})
    },
    last_px: 0,
    trades: Array(10).fill({px: 0, qty: 0, ts: 0})
});

let socket = new WebSocket("ws://localhost:7000/ws/marketdata");

socket.onmessage = (event) => {

    const msg = JSON.parse(event.data);

    // console.log(msg);

    market.set(msg);
};

socket.onclose = () => console.log("WebSocket disconnected");
socket.onerror = (err) => console.error("WebSocket error", err);
