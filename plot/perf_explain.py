import sys

import polars as pl
import polars.selectors as cs

import common


def main():
    df = (
        pl.scan_ndjson(sys.argv[1:])
        .select(
            common.SELECT_MAP,
            common.SELECT_TC,
            common.SELECT_KEY,
            common.SELECT_WORKLOAD,
            common.SELECT_OPS,
            common.SELECT_MEM,
            (common.SELECT_L3_HIT_HITM + common.SELECT_L3_MISS_HITM).alias("clt"),
            common.SELECT_BRANCH,
            common.SELECT_BRANCH_MISS,
        )
        .filter(pl.col("tc") == 80)
        .drop("tc")
        # .filter(pl.col("key") != common.Key.SEQ)
        .filter(pl.col("key") != common.Key.EMAIL)
        .filter(pl.col("key") != common.Key.URL)
        # .filter(pl.col("key") == common.Key.UUID_V4)
        .with_columns(
            mem=pl.col("mem")
            - pl.col("key").map_elements(
                lambda key: (
                    common.Key(key).memory_overhead()
                    + common.Key(key).memory_baseline()
                )
                / 2**30,
                returns_scalar=True,
                return_dtype=pl.Float64,
            )
        )
        .collect()
    )

    for wl, group in df.group_by("wl", maintain_order=True):
        for metric in [
            # ("l3", common.SELECT_L3_HIT + common.SELECT_L3_MISS),
            "mem",
            "clt",
            "branch",
            "branch_miss",
        ]:
            print(f"{wl}, {metric}")
            print(
                group.group_by(
                    "map",
                    "key",
                    "ops",
                )
                # .agg(pl.col(metric).sum() / pl.col("ops").sum())
                .agg(pl.col(metric).mean())
                .drop("ops")
                .pivot(index="key", on="map")
                # .with_columns(cs.exclude("key", "arctic") / pl.col("arctic"))
                # .with_columns(
                #     (cs.exclude("key", "arctic") / pl.col("arctic")).log().mean().exp()
                # )
                .select(
                    pl.col("key"),
                    pl.col("arctic")
                    # .mean()
                    .map_elements(common.display_abs, return_dtype=pl.String),
                    (cs.exclude("key", "arctic") ** -1 * pl.col("arctic"))
                    # (pl.col("arctic") / cs.exclude("key", "arctic") / pl.col("arctic"))
                    # .log()
                    # .mean()
                    # .exp()
                    .map_elements(common.display_rel, return_dtype=pl.String),
                )
            )


if __name__ == "__main__":
    main()
