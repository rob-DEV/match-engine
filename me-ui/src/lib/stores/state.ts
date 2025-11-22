import {writable} from "svelte/store";

// @ts-ignore
export const openOrders = writable<[]>([]);
// @ts-ignore
export const closedTrades = writable<[]>([]);