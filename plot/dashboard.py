import enum
from enum import Enum
import sys

import dash
from dash import Dash, html, dcc, Input, State, Output, callback
import dash_bootstrap_components as dbc
import plotly.express as px
import polars as pl
from polars import selectors as cs


DF = pl.read_ndjson(sys.argv[1]).with_columns(
    output=pl.col("output").struct.with_fields(
        thread=pl.field("thread").list.eval(
            pl.element().struct.with_fields(
                throughput=pl.field("operation_count") * 1e9 / pl.field("time"),
            )
        )
    )
)


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
        if self.aggregate:
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


def main():
    ui_control = [html.H2("Control")]
    ui_independent = [html.H2("Independent")]

    ui_process = [html.H2("Process")]
    ui_thread = [html.H2("Thread")]

    for col in flatten(DF):
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

        values = unique(col.selector)

        if len(values) == 1:
            value = values[0]
            if type(value) is bool:
                value = "true" if value else "false"
            elif value is None:
                value = NULL

            ui_control.append(
                dbc.Row(
                    [
                        dbc.Col(html.Span(col.name)),
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
                    dbc.Col(html.Span(col.name)),
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


def flatten(df):
    def recurse(columns, namespace, selector, aggregate):
        select = pl.col if selector is None else lambda col: selector.struct.field(col)

        for col in columns:
            dtype = df.select(select(col)).to_series().dtype
            name = col if namespace == "" else f"{namespace}/{col}"
            output = name.startswith("output")

            if hasattr(dtype, "fields"):
                # FIXME: more robust distribution detection
                if any([field.name == "p50" for field in dtype.fields]):
                    assert not aggregate
                    assert output
                    yield Col(name, select(col), distribution=True, output=output)
                    continue

                yield from recurse(
                    [field.name for field in dtype.fields],
                    name,
                    select(col),
                    aggregate,
                )

            # FIXME: only supports lists of structs, which
            # is true in our case (`output/thread`)
            elif hasattr(dtype, "inner"):
                yield from recurse(
                    [field.name for field in dtype.inner.fields],
                    name,
                    select(col).list.explode(),
                    True,
                )
            else:
                yield Col(
                    name,
                    select(col),
                    aggregate=aggregate,
                    output=output,
                )

    yield from recurse(df.columns, "", None, False)


def unique(selector):
    return DF.select(selector).unique().to_series().sort()


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
    if ts is None or any([value is None for value in values]):
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
        filtered = DF.filter(*filters) if len(filters) > 0 else DF

        sorts = [x.name]
        cols = [
            x.selector.first().alias(x.name),
        ]

        if op == "mean":
            cols.extend(
                [
                    y.selector.mean().alias(f"{y.name}"),
                    y.selector.std().alias(f"{y.name}_std"),
                ]
            )
        elif op == "sum":
            cols.append(
                y.selector.sum().alias(f"{y.name}"),
            )
        elif op == "show":
            cols.append(y.selector.alias(f"{y.name}"))

        else:
            assert False

        for col in [v for v in [facet_row, facet_column, facet_color] if v is not None]:
            sorts.append(col.name)
            if col.name not in filtered.columns:
                cols.append(col.selector.first().alias(col.name))

        filtered = filtered.group_by(cs.exclude("output", "date")).agg(cols).sort(sorts)

        if y.distribution:
            filtered = filtered.select(
                # Why is this a list[struct] and not just a struct?
                cs.exclude(y.name),
                pl.col(y.name).explode().struct.unnest(),
            ).unpivot(
                index=cs.exclude(["min", "max", "mean", "p50", "p75", "p90", "p99"]),
                variable_name="metric",
                value_name="value",
            )

        fig = px.line(
            filtered,
            x=x.name,
            y="value" if y.distribution else y.name,
            error_y=f"{y.name}_std" if op == "mean" else None,
            facet_row=facet_row.name if facet_row is not None else None,
            facet_col=facet_column.name if facet_column is not None else None,
            color="metric"
            if y.distribution
            else facet_color.name
            if facet_color is not None
            else None,
            markers=True,
            # log_y=True,
        )

        fig.update_xaxes(title_text=x.name, tickvals=filtered[x.name].unique())
        fig.update_yaxes(title_text=y.name, autorangeoptions_include=0.0)
        children.append(dcc.Graph(figure=fig))

    return children


if __name__ == "__main__":
    main()
