import sys

import polars as pl
import plotly.graph_objects as go
import plotly.subplots as sp
import polars.selectors as cs

import common
from common import bold

X_TITLE = bold("Thread Count")
Y_TITLE = bold("Throughput (Mops/sec) for Key Distribution")
YCSB = [wl for wl in common.Workload if wl.startswith("YCSB")]


def main():
    df = (
        pl.scan_ndjson(sys.argv[1:])
        .select(
            common.SELECT_MAP.alias("map"),
            common.SELECT_TC.alias("tc"),
            common.SELECT_KEY.alias("key"),
            common.SELECT_WORKLOAD.alias("wl"),
            common.SELECT_MEM.alias("mem"),
            common.SELECT_TP.alias("tp"),
        )
        .group_by(cs.exclude("tp", "mem"))
        .agg(
            pl.col("tp").mean(),
            pl.col("tp").std().fill_null(0).alias("tp_std"),
            pl.col("mem").mean(),
            pl.col("mem").std().fill_null(0).alias("mem_std"),
        )
        .with_columns(
            tp_std=pl.when((pl.col("tp_std") / pl.col("tp")) > 0.1).then("tp_std"),
            mem_std=pl.when((pl.col("mem_std") / pl.col("mem")) > 0.1).then("mem_std"),
        )
        .sort("key", "wl", "map")
        .collect()
    )

    fig = sp.make_subplots(
        rows=len(common.Key),
        cols=len(YCSB) + 1,
        shared_xaxes=True,
        subplot_titles=[bold(title) for title in list(YCSB) + ["YCSB-Load"]],
        # y_title=Y_TITLE,
        horizontal_spacing=0.025,
        vertical_spacing=0.015,
    )

    for (key,), row_data in df.group_by("key", maintain_order=True):
        i = common.Key(key).index() + 1

        for (wl,), col_data in row_data.group_by("wl", maintain_order=True):
            j = common.Workload(wl).index() + 1

            for (map,), map_data in col_data.group_by("map", maintain_order=True):
                map_data = map_data.sort("tc")
                map = common.Map(map)
                trace = go.Scatter(
                    x=map_data["tc"],
                    y=map_data["tp"],
                    error_y=dict(type="data", array=map_data["tp_std"]),
                    name=map,
                    legendgroup=map,
                    legendrank=map.index(),
                    **map.style(),
                )

                fig.add_trace(trace, i, j)

            fig.update_xaxes(
                rangemode="tozero",
                ticks="outside",
                tickformat="",
                ticklen=5,
                row=i,
                col=j,
                tick0=0,
                dtick=40,
                range=[0, 165],
            )
            fig.update_yaxes(rangemode="tozero", row=i, col=j)
            fig.add_vrect(
                type="rect",
                x0=80,
                x1=165,
                line_width=0,
                fillcolor="black",
                opacity=0.2,
                row=i,
                col=j,
            )

    map_count = len(df.select(pl.col("map").unique()))

    for (key,), row_data in (
        df.filter(pl.col("wl") == common.Workload.L, pl.col("tc") == 80)
        .with_columns(
            mem=(
                pl.col("mem")
                - pl.col("key").map_elements(
                    lambda key: common.Key(key).memory_overhead() / 2**30,
                    returns_scalar=True,
                    return_dtype=pl.Float64,
                )
            )
        )
        .with_columns(
            rel=pl.col("mem")
            / pl.col("key").map_elements(
                lambda key: common.Key(key).memory_baseline() / 2**30,
                returns_scalar=True,
                return_dtype=pl.Float64,
            )
        )
        .group_by("key", maintain_order=True)
    ):
        key = common.Key(key)

        i = key.index() + 1
        j = len(YCSB) + 1

        for (map,), map_data in row_data.group_by("map", maintain_order=True):
            map = common.Map(map)

            style = map.style()
            del style["marker"]["symbol"]
            del style["marker"]["size"]
            del style["line"]

            trace = go.Bar(
                x=[map],
                y=map_data["mem"],
                error_y=dict(type="data", array=map_data["mem_std"]),
                name=map,
                legendgroup=map,
                legendrank=map.index(),
                text=[f"{rel:.1f}x" for rel in map_data["rel"]],
                **style,
            )

            fig.add_trace(trace, i, j)
            fig.update_yaxes(
                side="right",
                row=i,
                col=j,
            )

        fig.update_xaxes(
            showticklabels=False,
            row=i,
            col=j,
            range=[-0.5, map_count - 0.5],
        )

        fig.add_hrect(
            type="rect",
            y0=0,
            y1=key.memory_baseline() / 2**30,
            line_width=0,
            fillcolor="black",
            opacity=0.2,
            row=i,
            col=j,
        )

    fig.update_xaxes(
        title=X_TITLE,
        row=len(common.Key),
        col=5,
    )

    fig.update_yaxes(
        title=bold("Peak Memory Usage (GiB)"),
        row=4,
        col=len(YCSB) + 1,
    )

    # Deduplicate legend entries
    # https://stackoverflow.com/a/62162555
    unique = set()
    fig.for_each_trace(
        lambda trace: trace.update(showlegend=False)
        if (trace.name in unique)
        else unique.add(trace.name)
    )

    for row, key in enumerate(common.Key):
        fig.update_yaxes(title=bold(key), row=row + 1, col=1)

    fig.update_layout(
        legend=dict(orientation="h", y=-0.04, title=bold("Index"), font=dict(size=16)),
        width=1080,
        height=700,
        margin=dict(l=0, r=0, t=20, b=0),
        uniformtext=dict(minsize=14, mode="show"),
    )
    # HACK: avoid overlap
    fig.update_annotations(selector=dict(text=Y_TITLE), xshift=-60)
    fig.write_image("ycsb.pdf")


if __name__ == "__main__":
    main()
