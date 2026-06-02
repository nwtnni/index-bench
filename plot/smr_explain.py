import sys
import polars as pl
import polars.selectors as cs

import common

pl.Config.set_tbl_cols(-1)

df = (
    pl.read_ndjson(sys.argv[1:])
    .select(
        common.SELECT_MAP,
        common.SELECT_TC,
        common.SELECT_BATCH,
        common.SELECT_KEY,
        common.SELECT_ZIPF,
        common.SELECT_UPDATE,
        common.SELECT_GARBAGE,
        common.SELECT_TP,
    )
    .filter(pl.col("zipf").is_in([0.99, 1.1, 1.2]))
    .group_by(cs.exclude("garbage", "tp"))
    .agg(garbage=pl.col("garbage").mean(), tp=pl.col("tp").mean())
    .with_columns(garbage=pl.col("garbage") / pl.col("tc"))
)

# 64 for crossbeam-epoch, 32 for seize
DEFAULT_BATCH_SIZE = 64

# Performance improvement from batch size configuration
print(
    df.filter(
        pl.col("map") != pl.lit(common.Map.ARCTIC_LEAK),
        pl.col("update") == 0.5,
        pl.col("tc") == 80,
    )
    .group_by(cs.exclude("batch", "garbage", "tp"))
    .agg(
        garbage=pl.col("garbage").filter(pl.col("batch") == 256)
        / pl.col("garbage").filter(pl.col("batch") == DEFAULT_BATCH_SIZE).get(0),
        tp=pl.col("tp").filter(pl.col("batch") == 256)
        / pl.col("tp").filter(pl.col("batch") == DEFAULT_BATCH_SIZE).get(0),
    )
    .explode("garbage", "tp")
    # Geomean across zipf
    .group_by(cs.exclude("zipf", "garbage", "tp"))
    .agg(
        garbage=pl.col("garbage").log().mean().exp(),
        tp=pl.col("tp").log().mean().exp(),
    )
    .drop_nulls()
    .with_columns(garbage=pl.col("garbage"))
)

# Throughput drop relative to EBR
print(
    df.drop("garbage")
    .filter(
        pl.col("zipf") == 0.99,
        pl.col("tc") == 80,
        pl.col("batch") == 256,
    )
    .group_by(cs.exclude("map", "tp"))
    .agg(
        map=pl.col("map"),
        tp=pl.col("tp")
        / pl.col("tp").filter(pl.col("map") == common.Map.ARCTIC_EBR).get(0),
    )
    .explode("map", "tp")
    .with_columns(tp=1 - pl.col("tp"))
    # .group_by("map", "update")
    # .agg(tp=pl.col("tp").log().mean().exp())
)
