# Concurrent

```js
const table = FileAttachment("./data/concurrent-load-thread.tsv").arquero();
```

```js
const base = table
    .derive({
        "config/workload/key": (d => {
            if (d["config/workload/key"] === "u64") {
                if (d["config/workload/insert_order"] === "hashed") {
                    return "rand-u64";
                } else {
                    return "seq-u64";
                }
            } else {
                return d["config/workload/key"];
            }
        }),
        "throughput": d => d["output/thread/operation_count"] * 1e3 / d["output/thread/time"]
    })
    .groupby(aq.startswith("config"))
    .rollup({
        "throughput": aq.op.sum("throughput"),
    })
    .ungroup();
```

```js
const key_opt = base
    .rollup({ distinct: aq.op.array_agg_distinct("config/workload/key") })
    .array("distinct")[0];

const key = view(Inputs.radio(
    key_opt,
    {
        "label": "Key type",
        value: key_opt[0]
    }
))
```

```js
const record_count_opt = base
    .params({ key: key })
    .filter((d, $) => d["config/workload/key"] === $.key)
    .rollup({ distinct: aq.op.array_agg_distinct("config/workload/record_count") })
    .array("distinct")[0];

const record_count = view(Inputs.radio(
    record_count_opt,
    {
        "label": "Record count",
        value: record_count_opt[record_count_opt.length - 1],
    }
))
```

```js
const df = base
    .params({
        record_count: record_count,
        key: key
    })
    .filter((d, $) => d["config/workload/record_count"] === $.record_count)
    .filter((d, $) => d["config/workload/key"] === $.key);
```

```js
Plot.plot({
    grid: true,
    symbol: { legend: true },
    width: width,
    height: 800,
    marks: [
        Plot.lineY(df, {
            x: "config/global/thread_count",
            y: "throughput",
            stroke: "config/index/name",
        }),
        Plot.dot(df, {
            x: "config/global/thread_count",
            y: "throughput",
            stroke: "config/index/name",
            symbol: "config/index/name",
        })
    ]
})
```
