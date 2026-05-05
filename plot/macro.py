import enum
import sys
import math

import polars as pl
import plotly.graph_objects as go
import polars.selectors as cs
import plotly.express as px

import common

X_TITLE = "Number of Threads"
Y_TITLE = "Throughput (rows/sec)"


def main():
    turso = pl.read_csv(sys.argv[1])
    rocksdb = pl.read_csv(
        sys.argv[2],
        has_header=True,
        separator="\t",
        comment_prefix="#",
    )

    turso = (
        turso.rename(dict(throughput="tp", threads="tc"))
        .filter(pl.col("compute") == 0, pl.col("batch_size") == 100)
        .group_by("index", "tc")
        .agg(
            pl.col("tp").mean(),
            pl.col("tp").std().alias("std"),
        )
        .select(
            wl=pl.lit("turso"),
            index=pl.col("index").str.replace("skipmap", "skiplist").cast(common.Map),
            tc="tc",
            tp="tp",
            std="std",
        )
    )

    rocksdb = (
        rocksdb.rename(dict(ops_sec="tp", memtablerep="index", num_threads="tc"))
        .group_by("index", "tc")
        .agg(pl.col("tp").mean(), pl.col("tp").std().alias("std"))
        .select(
            wl=pl.lit("rocksdb"),
            index=pl.col("index")
            # HACK: should ideally strip this when collecting data
            .str.replace("PERF_CONTEXT:", "")
            .str.replace("skip_list", "skiplist")
            .cast(common.Map),
            tc="tc",
            tp="tp",
            std="std",
        )
    )

    df = pl.concat([turso, rocksdb])

    for (wl,), group in df.sort("wl").group_by("wl", maintain_order=True):
        for (index,), row in group.sort("index").group_by("index", maintain_order=True):
            buffer = rf"\textbf{{{index}}}"

            for (tc,), col in row.sort("tc").group_by("tc", maintain_order=True):
                tp = col.select("tp").item()

                if index == common.Map.SKIPLIST:
                    tp = common.display_abs(tp)
                else:
                    baseline = (
                        df.filter(
                            pl.col("index") == pl.lit(common.Map.SKIPLIST),
                            pl.col("wl") == pl.lit(wl),
                            pl.col("tc") == pl.lit(tc),
                        )
                        .select("tp")
                        .item()
                    )
                    tp = common.display_rel(tp / baseline)

                buffer += f" & {tp}"

            buffer = buffer + r" \\ \hline"
            print(buffer)


if __name__ == "__main__":
    main()
