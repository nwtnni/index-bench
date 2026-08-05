import json

import click
import polars as pl
from polars import selectors as cs

import common


@click.command
@click.argument(
    "paths",
    nargs=-1,
    type=click.Path(
        exists=True,
        file_okay=True,
        dir_okay=False,
        readable=True,
        resolve_path=True,
    ),
)
def main(paths):
    # NOTE have to manually handle config/workload/request_distribution
    # because polars is not good at handling this enum encoding:
    # `sed -i 's/"uniform"/{"zipfian":0.0}/' ycsb-load.ndjson`
    # `sed -i 's/latest/zipfian/g' ycsb-d.ndjson`
    df = pl.concat(
        [pl.scan_ndjson(path) for path in paths],
        how="diagonal_relaxed",
    )

    df.select(
        *[
            col.selector.alias(col.name)
            for col in flatten(df)
            if (not col.aggregate or col.distribution)
            and col.name not in ["config/workload/key"]
        ],
        common.SELECT_KEY.alias("config/workload/key"),
        common.SELECT_TP.alias("output/throughput"),
    ).drop(
        "date",
        cs.starts_with("output/index"),
        "output/garbage",
        "output/memory_key_value",
        "config/workload/operation_count",
        "config/workload/insert_order",
    ).collect().write_csv("data.tsv", separator="\t")


def load(path):
    with open(path) as file:
        data = file.read()
        buffer = []
        for line in data.splitlines():
            if len(line) == 0:
                continue
            buffer.append(json.loads(line))
        return buffer


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

            # HACK: assume list of structs is thread output
            case pl.List(inner=pl.Struct(fields=fields)):
                for field in fields:
                    yield from recurse(
                        field.name,
                        field.dtype,
                        namespace,
                        lambda inner: selector(name).list.eval(
                            pl.element().struct.field(inner)
                        ),
                        True,
                    )
            # HACK: assume list of integers is numa/interleave
            case pl.List(inner=pl.Int64()):
                pass

            case pl.List():
                raise NotImplementedError("unrecognized list field")

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


class Col:
    def __init__(
        self,
        name: str,
        selector,
        output=False,
        aggregate=False,
        distribution=False,
    ):
        self.name = name
        self.selector = selector
        self.output = output
        self.aggregate = aggregate
        self.distribution = distribution


if __name__ == "__main__":
    main()
