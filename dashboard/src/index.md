# index-bench

Experiments run on a [Chameleon](https://www.chameleoncloud.org/) compute_icelake_r650 instance ([example](https://www.chameleoncloud.org/hardware/node/sites/tacc/clusters/chameleon/nodes/dde004bf-b99b-4c0a-b2d4-d5537378626a/)).
Each instance has two Intel(R) Xeon(R) Platinum 8380 CPUs, each with:
- 2.30 GHz
- 40 cores
- 120 MiB LLC
- 128 GiB DDR4 3200 DRAM

```js
import { decompressSync, strFromU8 } from "npm:fflate";
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
        const distinct = base.rollup({ distinct: aq.op.array_agg_distinct(name) }).columnAt(0)[0]
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

function load(id, label) {
    return JSON.parse(window.localStorage.getItem(id.concat("/", label)));
}

function store(id, label, value) {
    window.localStorage.setItem(id.concat("/", label), JSON.stringify(value));
}
```

<details><summary><h2 style="display: inline-block">Control variables</h2></summary>

```js
Inputs.table(control, { select: false })
```

</details>

<details open><summary><h2 style="display: inline-block">Independent variables</h2></summary>

```js
const x = view(Inputs.select(
    Object.keys(config),
    { label: "X-axis", value: load(id, "x") ?? "config/global/thread_count", sort: true }
))

const color = view(Inputs.select(
    Object.keys(config),
    { label: "Color", value: load(id, "c") ?? "config/index/name", sort: true }
))

const facet_x = view(Inputs.select(
    Object.keys(config).concat([null]),
    { label: "Facet X", value: load(id, "fx"), sort: true }
))

const facet_y = view(Inputs.select(
    Object.keys(config).concat([null]),
    { label: "Facet Y", value: load(id, "fy"), sort: true }
))
```

</details>

<details open><summary><h2 style="display: inline-block">Filter independent variables</h2></summary>

```js
const DEFAULTS = {
    "config/workload/load": [false],
    "config/workload/record_count": [100000000, 30000000],
    "config/workload/read_proportion": [0.95],
    "config/workload/update_proportion": [0.05],
    "config/workload/insert_proportion": [0.0],
    "config/workload/request_distribution/zipfian": [0.99],
    "config/workload/key": ["rand-u64"],
};

const inputs = {};
for (const [key, values] of Object.entries(config)) {
    const multiple = [x, color, facet_x, facet_y].includes(key);
    const input = multiple ? Inputs.checkbox : Inputs.radio;
    const defaults = DEFAULTS[key] ?? values;
    const value = load(id, key) ?? defaults;

    inputs[key] = input(values, {
        label: key,
        value: multiple ? value : value[0],
        sort: true
    });
}
const filters = view(Inputs.form(inputs));
```

</details>

<details open><summary><h2 style="display: inline-block">Dependent variables</h2></summary>

```js
const radios = {}
for (const key of output) {
    radios[key] = Inputs.radio(
        ["linear", "log", "hide"],
        {
            label: key,
            value: load(id, key) ?? (key === "output/throughput" ? "linear" : "hide"),
            sort: true,
            format: x => x ?? "Hide",
        }
    );
}

const ys = view(Inputs.form(radios));
```

</details>

## Plots

```js
let df = base;

for (const [key, value] of Object.entries(filters)) {
    const multiple = [x, color, facet_x, facet_y].includes(key);
    const values = multiple ? value : [value];

    store(id, key, values);

    df = df.params({ key, values }).filter((d, $) => {
        return aq.op.includes($.values, d[$.key]);
    });
}

store(id, "x", x);
store(id, "ys", ys);
store(id, "c", color);
store(id, "fx", facet_x);
store(id, "fy", facet_y);

for (const [y, scale] of Object.entries(ys)) {
    if (scale === "hide") {
        continue;
    }

    const marks = x === color ? [
            Plot.barY(df, {
                x,
                fx: facet_x,
                fy: facet_y,
                fill: color,
                tip: true,
                // HACK: https://observablehq.com/@tschaub/bar-plot-with-log-scale
                ...(scale == "Log" ? { y1: 1, y2: y } : { y }),
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

    display(html`
        <details open>
            <summary>
                <h3 style="display: inline-block">${ y }</h3>
            </summary>

            ${
                Plot.plot({
                    grid: true,
                    symbol: { legend: true },
                    width: width,
                    height: 1000,
                    x: {
                        tickRotate: -45,
                    },
                    y: {
                        tickFormat: ".2g",
                        type: scale,
                    },
                    marginLeft: 100,
                    marginBottom: 100,
                    marks,
                })
            }
        </details>
    `);

}
```
