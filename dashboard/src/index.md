# index-bench

```js
import { decompressSync, strFromU8 } from "npm:fflate";
import * as core from "./components/core.js";
```

```js
const compressed = await FileAttachment("./data/data.tsv.gz").arrayBuffer();
const decompressed = strFromU8(decompressSync(new Uint8Array(compressed)))
const base = aq.fromCSV(decompressed, { delimiter: "\t" })

const control = []
const config = {}
const output = []

for (const name of base.columnNames()) {
    if (name.startsWith("config")) {
        const distinct = core.distinct(base, name)
        if (distinct.length == 1) {
            control.push({ key: name, value: distinct[0] });
        } else {
            config[name] = distinct;
        }
    } else {
        output.push(name);
    }
}

const id = "default";
```

## Control

```js
Inputs.table(control, { select: false })
```

## Filter

```js
const checkboxes = {};
const DEFAULTS = {
    "config/workload/load": [false],
    "config/workload/record_count": [30000000, 100000000],
    "config/workload/read_proportion": [0.95],
    "config/workload/update_proportion": [0.05],
    "config/workload/insert_proportion": [0.0],
    "config/workload/request_distribution/zipfian": [0.99],
    "config/workload/key": ["rand-u64"],
};

for (const [key, values] of Object.entries(config)) {

    checkboxes[key] = core.load(id, key, Inputs.checkbox(values, { label: key, value: DEFAULTS[key] ?? values, sort: true }));
}
const filters = view(Inputs.form(checkboxes));
```

## Configure

```js
const x = view(core.load(id, "x", Inputs.select(
    Object.keys(config),
    { label: "X-axis", value: "config/global/thread_count", sort: true }
)))

const ys = view(core.load(id, "ys", Inputs.checkbox(
    output,
    { label: "Y-axis", value: ["output/throughput"], sort: true }
)));

const color = view(core.load(id, "c", Inputs.select(
    Object.keys(config),
    { label: "Color", value: "config/index/name", sort: true }
)))

const facet_x = view(core.load(id, "fx", Inputs.select(
    Object.keys(config).concat([null]),
    { label: "Facet X", value: null, sort: true }
)))

const facet_y = view(core.load(id, "fy", Inputs.select(
    Object.keys(config).concat([null]),
    { label: "Facet Y", value: null, sort: true }
)))
```

## Plot

```js
let df = base;

for (const [key, values] of Object.entries(filters)) {
    core.store(id, key, values);

    df = df.params({ key, values }).filter((d, $) => {
        return aq.op.includes($.values, d[$.key]);
    });
}

core.store(id, "x", x);
core.store(id, "ys", ys);
core.store(id, "c", color);
core.store(id, "fx", facet_x);
core.store(id, "fy", facet_y);

for (const y of ys) {
    const marks = x === color ? [
            Plot.barY(df, {
                x,
                y,
                fx: facet_x,
                fy: facet_y,
                fill: color,
                tip: true
            }),
        ] : [
            Plot.lineY(df, {
                x,
                y,
                fx: facet_x,
                fy: facet_y,
                stroke: color,
            }),
            Plot.dot(df, {
                x,
                y,
                stroke: color,
                symbol: color,
                fx: facet_x,
                fy: facet_y,
                tip: true,
            })
        ];

    view(Plot.plot({
        grid: true,
        symbol: { legend: true },
        width: width,
        height: 1000,
        x: {
            tickRotate: -45,
        },
        y: {
            tickFormat: ".2g",
        },
        marginLeft: 100,
        marginBottom: 100,
        marks,
    }));
}
```
