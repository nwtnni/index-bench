import sys

import polars as pl
import plotly.graph_objects as go
import plotly.subplots as sp

import common

X_TITLE = "Thread Count"
Y_TITLE = "Throughput (Mops/sec)"


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
        .filter(pl.col("key") != common.Key.KMER)
        .drop_nulls()
        .collect()
    )

    fig = sp.make_subplots(
        rows=len(common.Key) - 1,
        cols=len(common.Workload) + 1,
        shared_xaxes=True,
        subplot_titles=list(common.Workload) + ["YCSB-Load"],
        y_title=Y_TITLE,
        horizontal_spacing=0.02,
        vertical_spacing=0.02,
    )

    for (key,), row_data in df.group_by("key"):
        i = common.Key(key).index() + 1

        for (wl,), col_data in row_data.group_by("wl"):
            j = common.Workload(wl).index() + 1

            for (map,), map_data in col_data.group_by("map"):
                map_data = map_data.sort("tc")
                map = common.Map(map)
                trace = go.Scatter(
                    x=map_data["tc"],
                    y=map_data["tp"],
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
            )
            fig.update_yaxes(rangemode="tozero", row=i, col=j)
            fig.add_vrect(
                type="rect",
                x0=80,
                x1=160,
                line_width=0,
                fillcolor="black",
                opacity=0.1,
                row=i,
                col=j,
            )

    for (key,), row_data in df.filter(
        pl.col("wl") == common.Workload.L, pl.col("tc") == 80
    ).group_by("key", maintain_order=True):
        key = common.Key(key)

        i = key.index() + 1
        j = len(common.Workload) + 1

        for (map,), map_data in row_data.group_by("map", maintain_order=True):
            map_data = map_data.sort("tc").with_columns(
                pl.col("mem") - key.memory_overhead() / 2**30
            )
            map = common.Map(map)

            style = map.style()
            del style["marker"]["symbol"]
            del style["marker"]["size"]
            del style["line"]

            trace = go.Bar(
                x=[map],
                y=map_data["mem"],
                name=map,
                legendgroup=map,
                legendrank=map.index(),
                **style,
            )

            fig.add_trace(trace, i, j)
            fig.update_yaxes(side="right", row=i, col=j)

        fig.add_hline(
            y=key.memory_baseline() / 2**30,
            line=dict(dash="dash", color="blue", width=3),
            row=i,
            col=j,
        )
        fig.update_xaxes(showticklabels=False, row=i, col=j)

    fig.update_xaxes(
        title=X_TITLE,
        row=len(common.Key) - 1,
        col=5,
    )

    fig.update_yaxes(
        title="Peak memory usage (GiB)",
        row=4,
        col=len(common.Workload) + 1,
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
        fig.update_yaxes(title=key, row=row + 1, col=1)

    fig.update_layout(
        legend=dict(orientation="h", y=-0.04, font=dict(size=16)),
        width=1080,
        height=700,
        margin=dict(l=0, r=0, t=20, b=0),
    )
    # HACK: avoid overlap
    fig.update_annotations(selector=dict(text=Y_TITLE), xshift=-60)
    fig.write_image("ycsb.pdf")


if __name__ == "__main__":
    main()
