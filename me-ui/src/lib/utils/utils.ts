export function nsToDateTime(ns: string | number): string {
    const nsBig = BigInt(ns);

    const msBig = nsBig / 1_000_000n;

    const ms = Number(msBig);

    return new Date(ms).toLocaleString();
}

export type NumericObject = Record<string, number>;

export function isZeroArray<T extends NumericObject>(arr: T[]): boolean {
    for (let i = 0; i < arr.length; i++) {
        const obj = arr[i];
        for (const key in obj) {
            if (obj[key] !== 0) return false;
        }
    }
    return true;
}


export function boundedRandomNumber(min: number, max: number): number {
    const minCeil = Math.ceil(min);
    const maxFloor = Math.floor(max);
    return Math.floor(Math.random() * (maxFloor - minCeil) + minCeil);
}