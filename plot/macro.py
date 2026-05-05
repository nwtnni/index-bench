import sys

import polars as pl
import plotly.graph_objects as go
import plotly.subplots as sp
import polars.selectors as cs
import plotly.express as px

import common

X_TITLE = "Number of Threads"
Y_TITLE = "Throughput (rows/sec)"


def main():
    turso = pl.read_csv(sys.argv[1])
    rocksdb = pl.read_csv(
        sys.argv[2],
        has_header=True,
        separator="\t",
        comment_prefix="#",
    )

    CYCLE_TO_NS = 1e9 / 3.4e9

    turso = (
        turso.rename(dict(throughput="tp", threads="tc"))
        .filter(pl.col("compute") == 0, pl.col("batch_size") == 100)
        .group_by("index", "tc")
        .agg(
            pl.col("tp").mean(),
            (
                pl.col("total", "commit", "insert", "exists")
                / pl.col("tc")
                * CYCLE_TO_NS
                / 1e9
            ).mean(),
        )
        .select(
            wl=pl.lit("turso"),
            index=pl.col("index").str.replace("skipmap", "skiplist").cast(common.Map),
            tc="tc",
            tp="tp",
            total="total",
            commit=pl.col("commit"),
            memtable=pl.col("insert") + pl.col("exists"),
        )
        .with_columns(other=pl.col("total") - pl.col("commit") - pl.col("memtable"))
    )

    rocksdb = (
        rocksdb.rename(
            dict(
                ops_sec="tp",
                memtablerep="index",
                num_threads="tc",
                write_memtable_time="memtable",
                write_thread_wait_nanos="batch",
            )
        )
        .group_by("index", "tc")
        .agg(cs.numeric().mean())
        .select(
            wl=pl.lit("rocksdb"),
            index=pl.col("index")
            # HACK: should ideally strip this when collecting data
            .str.replace("PERF_CONTEXT:", "")
            .str.replace("skip_list", "skiplist")
            .cast(common.Map),
            tc="tc",
            tp="tp",
            total="uptime",
            memtable=pl.col("memtable") / 1e9,
            batch=pl.col("batch") / 1e9,
        )
        .with_columns(other=pl.col("total") - pl.col("memtable") - pl.col("batch"))
    )

    df = pl.concat([turso, rocksdb], how="diagonal")

    for (wl,), group in df.sort("wl").group_by("wl", maintain_order=True):
        for (index,), row in group.sort("index").group_by("index", maintain_order=True):
            buffer = rf"\textbf{{{index}}}"

            for (tc,), col in row.sort("tc").group_by("tc", maintain_order=True):
                tp = col.select("tp").item()

                if index == common.Map.SKIPLIST:
                    tp = common.display_abs(tp)
                else:
                    baseline = (
                        df.filter(
                            pl.col("index") == pl.lit(common.Map.SKIPLIST),
                            pl.col("wl") == pl.lit(wl),
                            pl.col("tc") == pl.lit(tc),
                        )
                        .select("tp")
                        .item()
                    )
                    tp = common.display_rel(tp / baseline)

                buffer += f" & {tp}"

            buffer = buffer + r" \\ \hline"
            print(buffer)

    fig = sp.make_subplots(rows=1, cols=2, horizontal_spacing=0.06)

    colors = dict(
        other=common.COLORS[0],
        memtable=common.COLORS[2],
        insert=common.COLORS[0],
        batch=common.COLORS[3],
        commit=common.COLORS[1],
    )

    rocksdb = rocksdb.sort("tc", "index", descending=True)

    for i, metric in enumerate(
        [
            "memtable",
            "batch",
            "other",
        ]
    ):
        fig.add_trace(
            go.Bar(
                x=rocksdb[metric],
                y=[rocksdb["tc"], rocksdb["index"]],
                name=metric if metric != "memtable" else "index",
                legend="legend1",
                legendrank=2 - i,
                marker=dict(color=colors[metric]),
                orientation="h",
                text=list(map(common.display_abs, rocksdb[metric])),
            ),
            row=1,
            col=1,
        )

    turso = turso.sort("tc", "index", descending=True)

    for i, metric in enumerate(
        [
            "memtable",
            "commit",
            "other",
        ]
    ):
        fig.add_trace(
            go.Bar(
                x=turso[metric],
                y=[turso["tc"], turso["index"]],
                name=metric if metric != "memtable" else "index",
                legend="legend2",
                legendrank=3 - i,
                marker=dict(color=colors[metric]),
                orientation="h",
                text=list(map(common.display_abs, turso[metric])),
            ),
            row=1,
            col=2,
        )

    fig.update_yaxes(title="<b>Thread Count and Index</b>", row=1, col=1)
    fig.update_xaxes(title="<b>Time (sec)</b>")

    fig.update_layout(
        barmode="stack",
        height=300,
        width=1080,
        uniformtext=dict(minsize=14, mode="show"),
        legend1=dict(
            font=dict(size=15),
            title="<b>RocksDB Operation</b>",
            orientation="h",
            xref="paper",
            yref="paper",
            x=0.0,
            y=1.15,
        ),
        legend2=dict(
            font=dict(size=15),
            title="<b>Turso Operation</b>",
            orientation="h",
            xref="paper",
            yref="paper",
            x=0.53,
            y=1.15,
        ),
        margin=dict(l=100, r=0, t=0, b=0),
        # plot_bgcolor="white",
    )

    fig.write_image("macro.pdf")


if __name__ == "__main__":
    main()
