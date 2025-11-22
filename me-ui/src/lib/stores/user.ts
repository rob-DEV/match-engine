import {writable} from "svelte/store";
import {browser} from "$app/environment";
import {boundedRandomNumber} from "$lib/utils/utils.ts";

function createClientId() {
    let initial = 0;

    if (browser) {
        const stored = localStorage.getItem("clientId");
        if (stored) {
            initial = Number(stored);
        } else {
            initial = boundedRandomNumber(100, 1_000_000);
            localStorage.setItem("clientId", initial.toString());
        }
    }

    const {subscribe, set} = writable<number>(initial);

    return {
        subscribe,
        // optional method to reset ID
        reset: () => {
            if (!browser) return;
            const id = Math.floor(Math.random() * 999_999) + 1;
            localStorage.setItem("clientId", id.toString());
            set(id);
        }
    };
}

export const clientId = createClientId();
