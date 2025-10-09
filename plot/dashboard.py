import sys
import math
from pathlib import PurePath
from typing import Optional

import click
import dash
from dash import Dash, html, dcc, Input, State, Output, callback
import dash_bootstrap_components as dbc

# HACK: https://stackoverflow.com/questions/9059699/use-a-library-locally-instead-of-installing-it
# HdrHistogram_py is not packaged for nix, possibly because it builds a C extension.
# Just vendor and build locally for now.
sys.path.insert(0, "HdrHistogram_py")
from HdrHistogram_py.hdrh.histogram import HdrHistogram
import plotly.express as px
import polars as pl
from polars import selectors as cs


DF = None

NULL = "null"
TYPE_COL = "col"
TYPE_STORE = "store"
ID_FIGURE = "figure"

COLS = []
CHOICES_INDEPENDENT = [
    {"label": label, "value": value}
    for label, value in [
        ("Set as X-axis", "x"),
        ("Facet along row", "facet_row"),
        ("Facet along column", "facet_column"),
        ("Facet along color", "facet_color"),
        ("Ignore", "ignore"),
    ]
]


class Col:
    def __init__(
        self, name: str, selector, output=False, aggregate=False, distribution=False
    ):
        self.name = name
        self.selector = selector
        self.output = output
        self.aggregate = aggregate
        self.distribution = distribution

    def store(self):
        return dcc.Store(
            id={"type": TYPE_STORE, "index": self.name},
            storage_type="local",
        )

    # ID used in pattern matching callback
    # https://dash.plotly.com/pattern-matching-callbacks
    def id(self):
        return {"type": TYPE_COL, "index": self.name}

    def choices(self):
        if self.aggregate and not self.distribution:
            return [
                {"label": label, "value": value}
                for label, value in [
                    ("Mean", "mean"),
                    ("Sum", "sum"),
                    ("Hide", "ignore"),
                ]
            ]
        else:
            return [
                {"label": label, "value": value}
                for label, value in [
                    ("Show", "show"),
                    ("Hide", "ignore"),
                ]
            ]


@click.command
@click.option("--vary")
@click.argument(
    "paths",
    nargs=-1,
    type=click.Path(
        exists=True,
        file_okay=True,
        dir_okay=False,
        readable=True,
        resolve_path=True,
        path_type=PurePath,
    ),
)
def main(vary, paths):
    df = pl.concat([load(vary, path) for path in paths])

    ui_control = [html.H2("Control")]
    ui_independent = [html.H2("Independent")]

    ui_process = [html.H2("Process")]
    ui_thread = [html.H2("Thread")]

    for col in flatten(df):
        if col.output:
            COLS.append(col)
            choices = col.choices()

            ui = ui_thread if col.aggregate else ui_process
            ui.append(
                dbc.Row(
                    [
                        col.store(),
                        dbc.Col(
                            html.Span(
                                col.name.removeprefix("output/").removeprefix("thread/")
                            )
                        ),
                        dbc.Col(
                            dcc.RadioItems(
                                choices,
                                id=col.id(),
                                inline=True,
                                value=choices[-1]["value"],
                            ),
                        ),
                    ]
                )
            )
            continue

        values = unique(df, col.selector)

        if len(values) == 1:
            value = values[0]
            if type(value) is bool:
                value = "true" if value else "false"
            elif value is None:
                value = NULL

            ui_control.append(
                dbc.Row(
                    [
                        dbc.Col(html.Span(col.name.removeprefix("config/"))),
                        dbc.Col(dcc.Dropdown([value], value=value, disabled=True)),
                    ]
                )
            )
            continue

        COLS.append(col)
        ui_independent.append(
            dbc.Row(
                [
                    col.store(),
                    dbc.Col(html.Span(col.name.removeprefix("config/"))),
                    dbc.Col(
                        dcc.Dropdown(
                            CHOICES_INDEPENDENT
                            + [
                                {
                                    "label": f"Filter to {NULL if value is None else value}",
                                    "value": NULL if value is None else value,
                                }
                                for value in values.to_list()
                            ],
                            id=col.id(),
                            value=CHOICES_INDEPENDENT[-1]["value"],
                        )
                    ),
                ]
            )
        )

    global DF
    DF = df

    app = Dash(
        external_stylesheets=[dbc.themes.BOOTSTRAP],
    )

    app.layout = [
        # https://community.plotly.com/t/is-there-a-way-to-trigger-load-on-initial-page-load-only-and-not-every-time-a-change-is-made-to-the-page/57504/4
        html.Div(id="init"),
        dbc.Row(html.H1(sys.argv[1])),
        dbc.Row(html.Hr()),
        dbc.Row(
            [
                dbc.Col(ui_control),
                dbc.Col(ui_independent),
            ]
        ),
        dbc.Row(
            [
                dbc.Col(ui_process),
                dbc.Col(ui_thread),
            ]
        ),
        html.Div(id=ID_FIGURE),
    ]
    app.run(debug=True)


def flatten(df: pl.LazyFrame):
    def recurse(
        name: str, dtype: pl.DataType, namespace: list[str], selector, aggregate: bool
    ):
        namespace.append(name)
        output = namespace[0] == "output"

        match dtype:
            # FIXME: more robust distribution detection
            case pl.String if output:
                yield Col(
                    "/".join(namespace),
                    selector(name),
                    distribution=True,
                    aggregate=aggregate,
                    output=output,
                )
            case pl.Struct(fields=fields):
                for field in fields:
                    yield from recurse(
                        field.name,
                        field.dtype,
                        namespace,
                        lambda inner: selector(name).struct.field(inner),
                        aggregate,
                    )

            # FIXME: only supports lists of structs, which
            # is true in our case (`output/thread`)
            case pl.List(inner=pl.Struct(fields=fields)):
                for field in fields:
                    yield from recurse(
                        field.name,
                        field.dtype,
                        namespace,
                        lambda inner: selector(name).list.explode().struct.field(inner),
                        True,
                    )
            case _:
                yield Col(
                    "/".join(namespace),
                    selector(name),
                    aggregate=aggregate,
                    output=output,
                )

        namespace.pop()

    schema = df.collect_schema()
    for name, dtype in zip(schema.names(), schema.dtypes()):
        yield from recurse(name, dtype, [], pl.col, False)


def unique(df, selector):
    return df.select(selector).unique().sort(cs.all()).collect().to_series()


def load(vary: Optional[str], path: PurePath):
    name = pl.field("name")

    if vary is not None:
        suffix = "-" + path.name.removesuffix(".gz").removesuffix(".ndjson")
        name = (
            pl.when(pl.field("name") == vary)
            .then(pl.field("name") + suffix)
            .otherwise(pl.field("name"))
        )

    return pl.scan_ndjson(str(path)).with_columns(
        # Rename system variants
        config=pl.col("config").struct.with_fields(
            pl.field("index").struct.with_fields(
                name=name,
            )
        ),
        # Compute throughput per thread
        output=pl.col("output").struct.with_fields(
            thread=pl.field("thread").list.eval(
                pl.element().struct.with_fields(
                    throughput=pl.field("operation_count") * 1e9 / pl.field("time"),
                )
            )
        ),
    )


@callback(
    Output({"type": TYPE_COL, "index": dash.ALL}, "value"),
    Input(component_id="init", component_property="children"),
    State({"type": TYPE_STORE, "index": dash.ALL}, "data"),
)
def init_store(_, store):
    return store


@callback(
    Output({"type": TYPE_STORE, "index": dash.MATCH}, "data"),
    Input({"type": TYPE_COL, "index": dash.MATCH}, "value"),
    prevent_initial_call=True,
)
def sync_store(ui):
    return ui


@callback(
    Output(component_id=ID_FIGURE, component_property="children"),
    Input({"type": TYPE_STORE, "index": dash.ALL}, "modified_timestamp"),
    dash.State({"type": TYPE_STORE, "index": dash.ALL}, "data"),
)
def update(
    ts,
    values,
):
    if DF is None or ts is None or any([value is None for value in values]):
        raise dash.exceptions.PreventUpdate

    x = None
    ys = []
    facet_row = None
    facet_column = None
    facet_color = None
    filters = []

    # Validate
    for col, value in zip(COLS, values):
        if value == "ignore":
            continue

        if col.output:
            ys.append((col, value))
        else:
            if value == "x":
                if x is not None:
                    return {}
                x = col
            elif value == "facet_row":
                if facet_row is not None:
                    return {}
                facet_row = col
            elif value == "facet_column":
                if facet_column is not None:
                    return {}
                facet_column = col
            elif value == "facet_color":
                if facet_color is not None:
                    return {}
                facet_color = col
            elif value == NULL:
                filters.append(col.selector.is_null())
            elif value is not None:
                filters.append(col.selector == value)

    if x is None or len(ys) == 0:
        raise dash.exceptions.PreventUpdate

    children = []

    for y, op in ys:
        df = DF.filter(*filters) if len(filters) > 0 else DF

        sort = [x.name]
        aggregate_local = [
            x.selector.first().alias(x.name),
        ]
        aggregate_global = [
            cs.exclude(y.name).first(),
        ]

        aggregate_local_y = y.selector.alias(f"{y.name}")
        aggregate_global_y = [
            pl.col(y.name).mean(),
            pl.col(y.name).std().alias(f"{y.name}_std"),
        ]

        if op == "mean":
            explode = pl.col(y.name).explode()
            aggregate_global_y = [
                explode.mean(),
                explode.std().alias(f"{y.name}_std"),
            ]
        elif op == "sum":
            aggregate_local_y = aggregate_local_y.sum()
        elif y.distribution:
            aggregate_global_y = [
                pl.col(y.name)
                .explode()
                .map_batches(
                    decode_histograms, return_dtype=pl.Object, returns_scalar=True
                )
            ]
        else:
            assert op == "show"
            pass

        aggregate_local.append(aggregate_local_y)
        aggregate_global.extend(aggregate_global_y)

        for col in [v for v in [facet_row, facet_column, facet_color] if v is not None]:
            aggregate_local.append(col.selector.first().alias(col.name))
            sort.append(col.name)

        df: pl.DataFrame = (
            # Aggregate locally within a single experiment (date)
            df.group_by("config", "date")
            .agg(aggregate_local)
            # Aggregate globally across experiments with the same configuration
            .group_by("config")
            .agg(aggregate_global)
            .sort(sort)
            .collect()
        )

        fig = None

        if not y.distribution:
            fig = px.line(
                df,
                x=x.name,
                y=y.name,
                error_y=f"{y.name}_std",
                facet_row=facet_row.name if facet_row is not None else None,
                facet_col=facet_column.name if facet_column is not None else None,
                color=facet_color.name if facet_color is not None else None,
                markers=True,
                color_discrete_sequence=px.colors.qualitative.Light24,
                # log_y=True,
            )

            fig.update_xaxes(title_text=x.name, tickvals=df[x.name].unique())
            fig.update_yaxes(title_text=y.name, autorangeoptions_include=0.0)
        else:
            histograms = df.select(y.name).to_series()
            high = max([histogram.max_value for histogram in histograms])
            bins = high
            width = math.ceil(high / bins)

            df = (
                df.with_columns(
                    pl.col(x.name).cast(pl.String),
                    pl.col(y.name).map_elements(
                        lambda histogram: expand_histogram(width, histogram),
                        return_dtype=pl.List(pl.Struct(dict(x=pl.UInt64, y=pl.UInt64))),
                        returns_scalar=True,
                    ),
                )
                .explode(y.name)
                .unnest(y.name)
            )

            fig = px.bar(
                df,
                x="x",
                y="y",
                color=x.name,
                barmode="group",
            )

            fig.update_xaxes(title_text=y.name)
            fig.update_yaxes(title_text="Count", autorangeoptions_include=0.0)

        children.append(dcc.Graph(figure=fig))

    return children


def decode_histograms(series: pl.Series) -> HdrHistogram:
    decoded = HdrHistogram.decode(series.first())
    for encoded in series.slice(1):
        decoded.decode_and_add(encoded)
    return decoded


def expand_histogram(width, histogram: HdrHistogram) -> pl.Series:
    if histogram.get_total_count() == 0:
        return pl.Series([], dtype=pl.Struct(dict(x=pl.UInt64, y=pl.UInt64)))

    items = []
    for index, item in enumerate(histogram.get_linear_iterator(width)):
        x = index * width
        y = item.count_added_in_this_iter_step
        items.append(dict(x=x, y=y))
    return pl.Series(items, dtype=pl.Struct(dict(x=pl.UInt64, y=pl.UInt64)))


if __name__ == "__main__":
    main()
