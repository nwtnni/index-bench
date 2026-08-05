import * as aq from "npm:arquero";

export function distinct(df, col) {
    return df.rollup({ distinct: aq.op.array_agg_distinct(col) }).columnAt(0)[0]
}

export function load(id, label, input) {
    input.value = JSON.parse(window.localStorage.getItem(id.concat("/", label))) ?? input.value;
    return input;
}

export function store(id, label, value) {
    window.localStorage.setItem(id.concat("/", label), JSON.stringify(value));
}
