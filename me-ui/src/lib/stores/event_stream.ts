import {writable} from 'svelte/store';

import {closedTrades, openOrders} from "$lib/stores/state.ts";

export type EngineMessage =
    | {
    type: "ApiOrderAckResponse";
    client_id: number;
    instrument: string;
    order_id: number;
    side: string;
    px: number;
    qty: number;
    ack_time: number
}
    | {
    type: "ApiCancelOrderAckResponse";
    client_id: number;
    instrument: string;
    order_id: number;
    cancel_order_status: string;
    reason: string;
    ack_time: number
}
    | {
    type: "ApiExecutionReportResponse";
    client_id: number;
    instrument: string;
    order_id: number;
    fill_type: string;
    exec_px: number;
    exec_qty: number;
    exec_type: string;
    exec_ns: number
};

export type OutgoingMessage =
    | { type: "ApiOrderRequest"; client_id: number; instrument: string; side: "buy" | "sell"; px: number; qty: number; time_in_force: string }
    | { type: "ApiOrderCancelRequest"; client_id: number; instrument: string; order_id: number };

export const wsMessages = writable([]);
let ws: WebSocket;

export function connectWS(clientId: number) {
    ws = new WebSocket(`ws://localhost:8080/ws/event_stream/${clientId}`);

    ws.onopen = () => {
        console.log("Connected to WebSocket");
    };

    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data) as EngineMessage;
            handleEngineMessage(data)

        } catch (e) {
            console.error("Bad JSON:", e);
        }
    };

    ws.onclose = () => {
        console.log("WS closed — reconnecting...");
        setTimeout(() => connectWS(clientId), 1000);
    };

    setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: "Heartbeat" }));
        }
    }, 5000);
}

function handleEngineMessage(msg: EngineMessage) {
    console.log(msg);
    switch (msg.type) {
        case "ApiOrderAckResponse":
            // order ack
            openOrders.update(orders => [
                ...orders,
                {
                    order_id: msg.order_id,
                    client_id: msg.client_id,
                    instrument: msg.instrument,
                    side: msg.side as "buy" | "sell",
                    px: msg.px,
                    qty: msg.qty,
                }
            ]);
            break;

        case "ApiCancelOrderAckResponse":
            // TODO: cancel order ack
            openOrders.update(orders =>
                orders.filter(o => o.order_id !== msg.order_id)
            );
            break;

        case "ApiExecutionReportResponse":
            // remove / adjust resting order
            openOrders.update(orders =>
                orders.map(o => {
                    if (o.order_id === msg.order_id) {
                        const newQty = o.qty - msg.exec_qty;
                        return newQty > 0 ? {...o, qty: newQty} : null;
                    }
                    return o;
                }).filter(Boolean)
            );

            // add trade
            closedTrades.update(trades => [
                ...trades,
                {
                    order_id: msg.order_id,
                    client_id: msg.client_id,
                    instrument: msg.instrument,
                    side: "buy", // or msg.side if available
                    px: msg.exec_px,
                    qty: msg.exec_qty,
                }
            ]);
            break;
    }
}

export function sendJson(obj: OutgoingMessage) {
    if (ws?.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(obj));
    } else {
        console.warn("WS not open yet, cannot send");
    }
}