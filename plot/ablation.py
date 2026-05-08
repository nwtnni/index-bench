import sys

import polars as pl
import plotly.express as px

import common


def main():
    df = (
        pl.scan_ndjson(sys.argv[1:], infer_schema_length=None)
        .select(
            common.SELECT_MAP.alias("map"),
            common.SELECT_TC.alias("tc"),
            common.SELECT_KEY.alias("key"),
            common.SELECT_WORKLOAD.alias("wl"),
            common.SELECT_MEM.alias("mem"),
            common.SELECT_TP.alias("tp"),
        )
        .drop_nulls()
        .filter(pl.col("key").is_in([common.Key.SEQ, common.Key.RAND]))
        .collect()
    )

    fig = px.bar(
        df,
        x="wl",
        y="tp",
        color="map",
        facet_col="key",
        barmode="group",
        facet_col_spacing=0.04,
        color_discrete_sequence=common.COLORS,
        color_discrete_map={common.Map.ARCTIC_4: "black"},
    )

    fig.for_each_annotation(lambda a: a.update(text=a.text.removeprefix("key=")))
    fig.update_xaxes(title="")
    fig.update_yaxes(matches=None, showticklabels=True)
    fig.update_yaxes(title="Throughput (Mops/sec)", row=1, col=1)
    fig.update_layout(
        legend=dict(title="Optimization", orientation="h"),
        margin=dict(l=0, r=0, t=20, b=0),
        height=300,
        width=400,
    )
    fig.write_image("ablation.pdf")


if __name__ == "__main__":
    main()
