import { writable } from 'svelte/store';

export type EngineMessage =
    | { type: "ApiOrderAckResponse"; client_id: number; instrument: string; side: string; px: number; qty: number; ack_time: number }
    | { type: "ApiCancelOrderAckResponse"; client_id: number; instrument: string; order_id: number; cancel_order_status: string; reason: string; ack_time: number }
    | { type: "ApiExecutionReportResponse"; client_id: number; instrument: string; order_id: number; fill_type: string; exec_px: number; exec_qty: number; exec_type: string; exec_ns: number };

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

            // Narrow type using the `type` field
            switch (data.type) {
                case "ApiOrderAckResponse":
                    console.log("Order acknowledged:", data);
                    break;
                case "ApiCancelOrderAckResponse":
                    console.log("Cancel acknowledged:", data);
                    break;
                case "ApiExecutionReportResponse":
                    console.log("Trade executed:", data);
                    break;
            }

            wsMessages.update(list => [...list, data]);
        } catch (e) {
            console.error("Bad JSON:", e);
        }
    };

    ws.onclose = () => {
        console.log("WS closed — reconnecting...");
        setTimeout(() => connectWS(clientId), 1000);
    };
}

export function sendOrder(obj: OutgoingMessage) {
    if (ws?.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(obj));
    } else {
        console.warn("WS not open yet, cannot send");
    }
}