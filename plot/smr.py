import sys
import polars as pl
import plotly.graph_objects as go
import plotly.subplots as sp

import common

df = (
    pl.read_ndjson(sys.argv[1:])
    .select(
        common.SELECT_MAP,
        common.SELECT_TC,
        common.SELECT_KEY,
        common.SELECT_ZIPF,
        common.SELECT_UPDATE,
        common.SELECT_BATCH,
        common.SELECT_GARBAGE / common.SELECT_TC,
    )
    .filter(pl.col("batch") == 256)
    .filter(pl.col("map") != pl.lit(common.Map.ARCTIC_LEAK))
    .filter(pl.col("zipf").is_in([0.99, 1.1, 1.2]))
    # .with_columns(
    #     map=pl.when(pl.col("map") == pl.lit(common.Map.ARCTIC))
    #     .then(pl.lit("hazard"))
    #     .otherwise(pl.col("map"))
    # )
    .with_columns(
        rel=pl.when(
            pl.col("map") == pl.lit(common.Map.ARCTIC),
        )
        .then(
            pl.col("garbage").map_elements(
                common.display_abs, return_dtype=pl.String, returns_scalar=True
            )
        )
        .otherwise(
            (
                pl.col("garbage")
                / (
                    pl.col("garbage")
                    .filter(pl.col("map") == pl.lit(common.Map.ARCTIC))
                    .get(0)
                    .over(["tc", "key", "zipf", "update"])
                )
            ).map_elements(
                common.display_rel, return_dtype=pl.String, returns_scalar=True
            )
        )
    )
    .sort("update", "map", "zipf", "tc", descending=[True, True, False, True])
    .rename(dict(tc="threads"))
    .with_columns(threads=pl.col("threads").cast(pl.String))
)

fig = sp.make_subplots(
    rows=2,
    cols=3,
    column_titles=list(map(common.bold, ["Zipf 0.99", "Zipf 1.1", "Zipf 1.2"])),
    row_titles=list(map(common.bold, ["YCSB-A", "YCSB-B"])),
    # shared_xaxes=True,
    shared_yaxes=True,
    horizontal_spacing=0.03,
    vertical_spacing=0.05,
)
fig.update_layout(
    uniformtext=dict(minsize=14, mode="show"),
)
shown = set()

for i, ((update,), outer) in enumerate(df.group_by("update", maintain_order=True)):
    for j, ((map,), row) in enumerate(outer.group_by("map", maintain_order=True)):
        for k, ((zipf,), col) in enumerate(row.group_by("zipf", maintain_order=True)):
            for (threads,), inner in col.group_by("threads", maintain_order=True):
                show = map not in shown
                shown.add(map)
                fig.add_trace(
                    go.Bar(
                        name=map if map != common.Map.ARCTIC else "hazard",
                        y=inner["threads"],
                        x=inner["garbage"],
                        text=inner["rel"],
                        textangle=0,
                        textposition="outside"
                        if inner["garbage"][0] < 6e3
                        else "inside",
                        marker=dict(color=common.Map(map).style()["marker"]["color"]),
                        legendrank=common.Map(map).index(),
                        width=0.25,
                        orientation="h",
                        showlegend=show,
                    ),
                    row=i + 1,
                    col=(k + 1),
                )

# for i, ((wl,), group) in enumerate(df.group_by("update", maintain_order=True)):
#     wl = {0.5: "A", 0.05: "B"}[wl]
#     fig = px.bar(
#         group.sort(
#             "update", "map", "zipf", "threads", descending=[True, False, False, False]
#         ).with_columns(pl.col("map").str.replace("arctic", "hazard")),
#         x="map",
#         y="garbage",
#         color="map",
#         # pattern_shape="map",
#         facet_row="threads",
#         facet_col="zipf",
#         facet_row_spacing=0.03,
#         facet_col_spacing=0.01,
#         text="rel",
#         color_discrete_map=dict(
#             hazard="black",
#             epoch=common.COLORS[0],
#             hyaline=common.COLORS[1],
#         ),
#         # pattern_shape_map=dict(
#         #     epoch="/",
#         #     hyaline="+",
#         # ),
#     )
#
#     # https://community.plotly.com/t/quick-help-with-barchart-text-formatting/47808
#     # fig.update_traces(textangle=90, textposition="outside", selector=dict(type="bar"))
#     # fig.update_traces(
#     #     textposition="inside", selector=dict(type="bar", x="epoch"), row=1
#     # )
#
#     fig.update_yaxes(title=None)
#     fig.update_yaxes(title=common.bold("Maximum Unreclaimed Allocations"), row=2, col=1)
#
#     fig.update_layout(
#         title=common.bold(f"Reclamation efficiency on YCSB-{wl}"),
#         width=600,
#         height=400,
#         legend=dict(
#             orientation="h",
#             title=common.bold("Safe Memory Reclamation Scheme"),
#             x=-0.05,
#             y=-0.02,
#         ),
#         margin=dict(t=50, b=0, l=0, r=10),
#         uniformtext=dict(minsize=14, mode="show"),
#     )
#
#     fig.for_each_xaxis(lambda xaxis: xaxis.update(showticklabels=False, title=None))
#     fig.write_image(f"smr-{wl.lower()}.pdf")

fig.update_yaxes(**common.title("Thread Count"), col=1)
fig.update_xaxes(
    **common.title("Peak Unreclaimed Retired Allocations per Thread"), row=2, col=2
)
# HACK: match ycsb figure
fig.update_annotations(textangle=-90, selector=lambda a: "YCSB" in a.text)

fig.update_yaxes(range=[-0.5, 2.5])
fig.update_layout(
    barmode="group",
    bargroupgap=0.0,
    width=500,
    height=400,
    legend=dict(
        orientation="h",
        title=common.bold("SMR Scheme"),
        y=1.15,
        font=dict(size=16),
    ),
    uniformtext=dict(minsize=16, mode="show"),
    margin=dict(t=0, b=0, l=0, r=0),
)
fig.write_image("smr.pdf")
