import sys

import polars as pl
import polars.selectors as cs
import plotly.express as px

import common
from common import bold


def main():
    df = (
        pl.scan_ndjson(sys.argv[1:], infer_schema_length=None)
        .select(
            common.SELECT_MAP,
            common.SELECT_TC,
            common.SELECT_KEY,
            common.SELECT_WORKLOAD,
            common.SELECT_TP,
        )
        .filter(pl.col("map") != common.Map.ARCTIC_4)
        .group_by(cs.exclude("tp"), maintain_order=True)
        .agg(tp=pl.col("tp").mean())
        .collect()
    )

    df = (
        pl.concat(
            [
                df,
                df.pivot(on="map", values="tp")
                .with_columns(
                    pl.col(
                        common.Map.ARCTIC_0,
                        common.Map.ARCTIC_1,
                        common.Map.ARCTIC_2,
                        common.Map.ARCTIC_3,
                    )
                    / pl.col(common.Map.ARCTIC_0)
                )
                .unpivot(
                    index=["tc", "key", "wl"], variable_name="map", value_name="rel"
                )
                .cast(dict(map=pl.Enum(common.Map))),
            ],
            how="align",
        )
        .with_columns(
            rel=pl.when(pl.col("map") == common.Map.ARCTIC_0)
            .then(pl.col("tp").map_elements(common.display_abs, return_dtype=pl.String))
            .otherwise(
                pl.col("rel").map_elements(
                    lambda x: common.display_rel(x, sig=3), return_dtype=pl.String
                )
            )
        )
        .sort("key", "wl", "map", descending=[False, True, True])
        .with_columns(
            wl=pl.col("wl")
            .cast(pl.String)
            .str.replace(common.Workload.L, "Insert")
            .str.replace(common.Workload.C, "Read")
        )
    )

    fig = px.bar(
        df,
        x="tp",
        y="wl",
        color="map",
        text="rel",
        facet_col="key",
        barmode="group",
        facet_col_spacing=0.04,
        color_discrete_sequence=common.COLORS,
        color_discrete_map={common.Map.ARCTIC_3: "black"},
        orientation="h",
    )

    def flip(trace):
        trace.textposition = ["outside" if x < 250 else "inside" for x in trace.x]

    fig.for_each_trace(flip)

    fig.for_each_annotation(
        lambda a: a.update(
            text=bold(a.text.replace("key=", "Key ")), font=dict(size=16)
        )
    )

    fig.update_yaxes(title="")
    fig.update_yaxes(**common.title("Operation"), row=1, col=1)

    fig.update_xaxes(title="", matches=None, showticklabels=True)
    fig.update_xaxes(**common.title("Throughput (Mops/sec)"))
    fig.update_layout(
        legend=dict(
            title=bold("Optimization"),
            orientation="h",
            # x=-0.1,
            y=1.15,
            traceorder="reversed",
            font=dict(size=16),
        ),
        margin=dict(l=0, r=0, t=0, b=0),
        uniformtext=dict(minsize=16, mode="show"),
        width=500,
        height=400,
    )
    fig.write_image("ablation.pdf")


if __name__ == "__main__":
    main()
