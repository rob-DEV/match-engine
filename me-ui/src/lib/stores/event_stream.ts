import {writable} from 'svelte/store';

export interface EngineMessage {
    type: string;
    data: any;
}

// Central store for incoming engine messages
export const wsMessages = writable<EngineMessage[]>([]);

let ws: WebSocket | null = null;

export function wsConnectClient(clientId: number) {
    if (ws) return;

    ws = new WebSocket(`ws://localhost:8080/ws/event_stream/${clientId}`);

    ws.onopen = () => {
        console.log('WS connected');
    };

    ws.onmessage = (event) => {
        const msg: EngineMessage = JSON.parse(event.data);
        wsMessages.update((arr) => [...arr, msg]);
    };

    ws.onclose = () => {
        console.log('WS disconnected, reconnecting in 1s');
        ws = null;
        setTimeout(() => wsConnectClient(clientId), 1000);
    };

    ws.onerror = (err) => {
        console.error('WS error', err);
        ws?.close();
    };
}

export function sendOrder(order: any) {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    ws.send(JSON.stringify(order));
}