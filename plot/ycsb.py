import sys

import polars as pl
import plotly.graph_objects as go
import plotly.subplots as sp
import polars.selectors as cs

import common
from common import bold

YCSB = [wl for wl in common.Workload if wl.startswith("YCSB")]

FONT_SIZE_TITLE = 18  # 24
FONT_SIZE_LEGEND = 16  # 20
FONT_SIZE = 14  # 18
WIDTH = 1080  # 2000 for slide
HEIGHT = 700  # 200 * key for slide


def main():
    df = (
        pl.scan_ndjson(sys.argv[1:])
        .select(
            common.SELECT_MAP,
            common.SELECT_TC,
            common.SELECT_KEY,
            common.SELECT_WORKLOAD,
            common.SELECT_MEM,
            common.SELECT_TP,
        )
        # .filter(pl.col("key").is_in([common.Key.SEQ, common.Key.RAND]))
        # .filter(
        #     pl.col("key").is_in(
        #         [common.Key.IP, common.Key.SNOWFLAKE, common.Key.UUID_V4]
        #     )
        # )
        # .filter(pl.col("key").is_in([common.Key.EMAIL, common.Key.URL]))
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

    key_count = len(df.select(pl.col("key").unique()))
    map_count = len(df.select(pl.col("map").unique()))

    fig = sp.make_subplots(
        rows=key_count,
        # cols=len(YCSB),
        cols=len(YCSB) + 1,
        shared_xaxes=True,
        column_titles=[bold(title) for title in list(YCSB) + ["YCSB-Load"]],
        # column_titles=[bold(wl + f" ({wl.description()})") for wl in list(YCSB)],
        horizontal_spacing=0.025,
        vertical_spacing=0.015,
    )

    for i, ((key,), row_data) in enumerate(df.group_by("key", maintain_order=True)):
        i = i + 1

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

    for i, ((key,), row_data) in enumerate(
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
        max_mem = row_data.select("mem").max().item()

        i = i + 1
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
                text=common.display_rel(map_data["rel"].item()),
                textangle=-90,
                textposition="inside"
                if map_data["mem"].item() / max_mem > 0.5
                else "outside",
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
        **common.title("Thread Count"),
        row=key_count,
        col=len(YCSB),
        title_font_size=FONT_SIZE_TITLE,
    )

    fig.update_yaxes(
        **common.title("Peak Memory Usage (GiB)"),
        row=key_count // 2 + 1,
        col=len(YCSB) + 1,
        title_font_size=FONT_SIZE_TITLE,
    )

    # Deduplicate legend entries
    # https://stackoverflow.com/a/62162555
    unique = set()
    fig.for_each_trace(
        lambda trace: trace.update(showlegend=False)
        if (trace.name in unique)
        else unique.add(trace.name)
    )

    for row, key in enumerate(
        df.select(pl.col("key")).unique(maintain_order=True).to_series()
    ):
        fig.update_yaxes(
            **common.title(key), row=row + 1, col=1, title_font_size=FONT_SIZE_TITLE
        )

    fig.update_layout(
        legend=dict(
            orientation="h",
            y=-0.04,
            title=bold("Index"),
            font=dict(size=FONT_SIZE_LEGEND),
        ),
        width=WIDTH,
        height=HEIGHT,
        margin=dict(l=0, r=0, t=20, b=0),
        uniformtext=dict(minsize=FONT_SIZE, mode="show"),
        font=dict(size=FONT_SIZE),
    )
    # https://community.plotly.com/t/setting-subplot-title-font-sizes/46612/2
    fig.update_annotations(font_size=FONT_SIZE_TITLE)
    fig.write_image("ycsb.pdf")


if __name__ == "__main__":
    main()
