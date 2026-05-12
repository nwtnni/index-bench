import sys
import polars as pl
import plotly.express as px

import common

df = (
    pl.read_ndjson(sys.argv[1:])
    .select(
        common.SELECT_MAP.alias("map"),
        common.SELECT_TC,
        common.SELECT_KEY,
        common.SELECT_ZIPF,
        common.SELECT_UPDATE,
        common.SELECT_GARBAGE.alias("garbage"),
    )
    .filter(common.approx_eq(pl.col("update"), 0.95))
    .filter(pl.col("zipf").is_between(0.99, 1.1))
    .filter(pl.col("tc").is_between(60, 100))
    .filter(pl.col("map") != pl.lit(common.Map.ARCTIC_LEAK))
    .with_columns(
        map=pl.when(pl.col("map") == pl.lit(common.Map.ARCTIC))
        .then(pl.lit("hazard"))
        .otherwise(pl.col("map"))
    )
    .rename(dict(tc="threads"))
)

print(df)
fig = px.bar(
    df,
    x="map",
    y="garbage",
    color="map",
    facet_row="threads",
    facet_col="zipf",
    facet_row_spacing=0.03,
    facet_col_spacing=0.07,
)

fig.update_yaxes(matches=None)
fig.for_each_yaxis(lambda yaxis: yaxis.update(showticklabels=True))
fig.update_yaxes(title=None)
fig.update_yaxes(title=common.bold("Maximum unreclaimed allocations"), row=2, col=1)

fig.update_layout(
    title=dict(text=common.bold("YCSB-B reclamation efficiency")),
    width=500,
    height=400,
    legend=dict(
        orientation="h",
        title=common.bold("Safe memory reclamation scheme"),
        x=-0.05,
        y=-0.02,
    ),
    margin=dict(t=50, b=0, l=0, r=10),
)

fig.for_each_xaxis(lambda xaxis: xaxis.update(showticklabels=False, title=None))
fig.write_image("smr.pdf")
