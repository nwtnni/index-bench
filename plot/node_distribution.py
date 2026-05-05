import sys

# HACK: https://stackoverflow.com/questions/9059699/use-a-library-locally-instead-of-installing-it
# HdrHistogram_py is not packaged for nix, possibly because it builds a C extension.
# Just vendor and build locally for now.
sys.path.insert(0, "HdrHistogram_py")
from HdrHistogram_py.hdrh.histogram import HdrHistogram
import polars as pl
import plotly.graph_objects as go

import common


def main():
    node_distributions()


def node_distributions():
    df = pl.read_ndjson(sys.argv[1:])

    df = df.select(
        common.SELECT_KEY.alias("key"),
        pl.col("output").struct["index"].struct["node_3"],
        pl.col("output").struct["index"].struct["node_15"],
        pl.col("output").struct["index"].struct["node_47"],
        pl.col("output").struct["index"].struct["node_256"],
    )

    keys = []
    nodes = []
    bins = []
    counts = []

    for (key,), group in df.group_by("key"):
        for n in [3, 15, 47, 256]:
            decoded = HdrHistogram.decode(group[f"node_{n}"].item())
            iterator = decoded.get_recorded_iterator()
            for _ in iterator:
                x = iterator.value_at_index
                y = iterator.count_at_this_value

                keys.append(key)
                nodes.append(f"Node{n}")
                bins.append(x)
                counts.append(y)

    df = (
        pl.DataFrame(dict(key=keys, node=nodes, bin=bins, count=counts))
        .cast(dict(key=common.Key, node=common.Node))
        .group_by("key", "node")
        .agg(pl.col("count").sum())
    )

    print(df.group_by("key", maintain_order=True).agg(pl.col("count").sum()))

    df = (
        pl.concat(
            [
                partition.with_columns(rel=pl.col("count") / pl.col("count").sum())
                for partition in df.partition_by("key", maintain_order=True)
            ]
        )
        .with_columns(
            rel_text=pl.when(pl.col("rel") > 0.1)
            .then(
                (pl.col("rel") * 100).map_elements(
                    "{:.0f}%".format, return_dtype=pl.String
                )
            )
            .otherwise(None),
        )
        .filter(pl.col("key") != common.Key.KMER)
        .sort("key", "node", descending=[True, False])
    )

    fig = go.Figure()

    xs = df.select("key").unique(maintain_order=True).to_series(0)

    for index, ((node,), group) in enumerate(df.group_by("node", maintain_order=True)):
        node = common.Node(node)
        fig.add_trace(
            go.Bar(
                name=f"<b>{node}</b>",
                y=xs,
                x=group.select("rel").to_series(0),
                orientation="h",
                text=group.select("rel_text").to_series(0),
                textfont_color="black",
                legendrank=3 - index,
                marker_pattern_shape=node.pattern(),
                marker=dict(pattern=dict(size=5, solidity=0.1)),
            )
        )

    fig.update_traces(marker=dict(line_color="black", pattern_fillmode="replace"))
    fig.update_xaxes(
        title=dict(text="<b>Node Proportion</b>"),
        range=[0, 1],
        showticklabels=False,
        showgrid=False,
    )
    fig.update_yaxes(
        title="<b>Key Distribution</b>",
        showgrid=False,
    )
    fig.update_layout(
        barmode="stack",
        height=150,
        width=400,
        legend=dict(orientation="h", x=-0.1, y=1.3, title=""),
        margin=dict(l=0, r=0, t=0, b=0),
        uniformtext=dict(minsize=16, mode="show"),
        plot_bgcolor="white",
    )
    fig.write_image("node-distribution.pdf")


if __name__ == "__main__":
    main()
