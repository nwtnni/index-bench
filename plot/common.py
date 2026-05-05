import enum

import polars as pl

# HACK: for whatever reason I cannot import
# plotly.colors.qualitative...
#
# Maybe related: https://github.com/plotly/plotly.py/issues/1672
# https://github.com/plotly/plotly.py/issues/2281
#
# https://community.plotly.com/t/plotly-colours-list/11730/3
COLORS = [
    "#1f77b4",  # muted blue
    "#ff7f0e",  # safety orange
    "#2ca02c",  # cooked asparagus green
    "#d62728",  # brick red
    "#9467bd",  # muted purple
    "#8c564b",  # chestnut brown
    "#e377c2",  # raspberry yogurt pink
    "#7f7f7f",  # middle gray
    "#bcbd22",  # curry yellow-green
    "#17becf",  # blue-teal
]


class Node(enum.StrEnum):
    NODE_3 = "Node3"
    NODE_15 = "Node15"
    NODE_47 = "Node47"
    NODE_256 = "Node256"

    def pattern(self):
        match self:
            case Node.NODE_3:
                return "."
            case Node.NODE_15:
                return "/"
            case Node.NODE_47:
                return "\\"
            case Node.NODE_256:
                return "|"


class Map(enum.StrEnum):
    SKIPLIST = "skiplist"
    ARCTIC = "arctic"
    ART = "art"
    DM = "dash_map"
    FB = "fb_tree"
    # PP = "papaya"
    # SCC_HM = "scc_hash_map"
    # SCC_TI = "scc_tree_index"
    WH = "wormhole"
    CB = "crossbeam_skiplist"
    ARCTIC_LEAK = "arctic-leak"
    ARCTIC_EBR = "arctic-epoch"

    ARCTIC_0 = "baseline"
    ARCTIC_1 = "+path"
    ARCTIC_2 = "+int"
    ARCTIC_3 = "+simd"
    ARCTIC_4 = "+membarrier"

    def index(self):
        return list(Map).index(self)

    def style(self):
        index = self.index()
        color = COLORS[index]
        symbol = "diamond"

        match self:
            # Tries
            case Map.ARCTIC | Map.ART:
                if self == Map.ARCTIC:
                    color = "black"
                symbol = "triangle-up"
            # Hash tables
            # Map.PP | Map.SCC_HM:
            case Map.DM:
                symbol = "hexagon"
            # B+-trees
            # Map.SCC_TI
            case Map.FB:
                symbol = "bowtie"
            case Map.CB:
                symbol = "circle"
            # Other
            case _:
                pass

        style = dict(
            line=dict(color=color),
            marker=dict(color=color, symbol=symbol, size=7),
            zorder=len(Map) - index,
        )
        return style


class Workload(enum.StrEnum):
    L = "YCSB-Load"
    A = "YCSB-A"
    B = "YCSB-B"
    C = "YCSB-C"
    D = "YCSB-D"
    E = "YCSB-E"

    # SCAN = "Scan"
    # READ = "Read"

    def index(self):
        return list(Workload).index(self)


class Key(enum.StrEnum):
    SEQ = "seq-u64"
    RAND = "rand-u64"
    IP = "ipv4"
    SNOWFLAKE = "twitter"
    KMER = "kmer"
    EMAIL = "email"
    URL = "url"

    def index(self):
        return list(Key).index(self)

    # Memory used by input file
    def memory_overhead(self):
        match self:
            case Key.SEQ | Key.RAND:
                return 0
            case Key.KMER:
                # File size (`du SRR31218470.bin`)
                return 879308 * 1024
            case Key.EMAIL:
                return (
                    # File size (`du email.txt`)
                    2194892 * 1024
                    +
                    # Mapping from index to &str (`wc -l email.txt` * `sizeof(&str)`)
                    114195476 * 16
                )
            case Key.URL:
                return (
                    # File size (`du url.txt`)
                    2712200 * 1024
                    +
                    # Mapping from index to &str (`wc -l url.txt` * `sizeof(&str)`)
                    38343209 * 16
                )
            case Key.IP:
                # File size (`du ipv4.bin`)
                return 544208 * 1024
            case Key.SNOWFLAKE:
                # File size (`du snowflake.bin`)
                return 859380 * 1024

    # Computed from arctic summing key/value sizes after load
    # Field `output/memory_key_value` with feature flag `arctic/stat`
    def memory_baseline(self):
        match self:
            case Key.SEQ | Key.RAND:
                return 1612800000
            case Key.KMER:
                return 961790032
            case Key.EMAIL:
                return 1853303369
            case Key.URL:
                return 1799965861
            case Key.IP:
                return 1140147200
            case Key.SNOWFLAKE:
                return 1612777568


# Create a polars expression that translates a row into
# a symbolic name using the mapping in `translate`.
def translate(translate: dict[str, pl.Expr]) -> pl.Expr:
    expr = pl.when(False).then(pl.lit("ERROR"))
    for name, select in translate.items():
        expr = expr.when(select).then(pl.lit(name))
    return expr


def approx_eq(expr: pl.Expr, literal: float) -> pl.Expr:
    return expr.sub(literal).abs().lt(1e-5)


_NAME = pl.col("config").struct["index"].struct["name"]
_SMR = pl.col("config").struct["index"].struct["smr"]
SELECT_MAP = translate(
    {map.value: _NAME == map.value for map in Map if not map.value.startswith("arctic")}
    | {
        Map.ARCTIC: (_NAME == "arctic").and_(_SMR == "hazard"),
        Map.ARCTIC_LEAK: (_NAME == "arctic").and_(_SMR == "disable"),
        Map.ARCTIC_EBR: (_NAME == "arctic").and_(_SMR == "epoch"),
        Map.ARCTIC_0: (_NAME == "arctic-0"),
        Map.ARCTIC_1: (_NAME == "arctic-1"),
        Map.ARCTIC_2: (_NAME == "arctic-2"),
        Map.ARCTIC_3: (_NAME == "arctic-3"),
        Map.ARCTIC_4: (_NAME == "arctic-4"),
    }
)

_WL = pl.col("config").struct["workload"]
_ZF = _WL.struct["request_distribution"].struct.field("zipfian").is_not_null()

SELECT_WORKLOAD = translate(
    {
        Workload.L: _WL.struct["load"],
        Workload.A: approx_eq(_WL.struct["read_proportion"], 0.5).and_(_ZF),
        Workload.B: approx_eq(_WL.struct["read_proportion"], 0.95)
        .and_(approx_eq(_WL.struct["update_proportion"], 0.05))
        .and_(_ZF),
        Workload.C: approx_eq(_WL.struct["read_proportion"], 1.0).and_(_ZF),
        # FIXME: detect if `latest` field is not null instead of relying on early exit above
        Workload.D: approx_eq(_WL.struct["read_proportion"], 0.95).and_(
            approx_eq(_WL.struct["insert_proportion"], 0.05)
        ),
        Workload.E: approx_eq(_WL.struct["scan_proportion"], 0.95),
    }
)

_KEY = pl.col("config").struct["workload"].struct["key"]
_ORDER = pl.col("config").struct["workload"].struct["insert_order"]
SELECT_KEY = translate(
    {
        Key.SEQ: (_KEY == "u64").and_(_ORDER == "ordered"),
        Key.RAND: (_KEY == "u64").and_(_ORDER == "hashed"),
        Key.KMER: (_KEY == "kmer").and_(_ORDER == "ordered"),
        Key.EMAIL: (_KEY == "email").and_(_ORDER == "hashed"),
        Key.URL: (_KEY == "url").and_(_ORDER == "hashed"),
        Key.IP: (_KEY == "ipv4").and_(_ORDER == "hashed"),
        Key.SNOWFLAKE: (_KEY == "snowflake").and_(_ORDER == "ordered"),
    }
).cast(pl.Enum(Key))

SELECT_TC = pl.col("config").struct["global"].struct["thread_count"]

SELECT_MEM = (
    pl.col("output").struct["mimalloc"].struct["committed"].struct["peak"] / 1e9
)
SELECT_TP = (
    pl.col("output")
    .struct["thread"]
    .list.eval(
        pl.element().struct["operation_count"] * 1e9 / pl.element().struct["time"]
    )
    .list.sum()
    / 1e6
)
SELECT_L3_HIT = pl.col("output").struct["perf"].struct["l3_hit"]
SELECT_L3_MISS = pl.col("output").struct["perf"].struct["l3_miss"]
SELECT_BRANCH = pl.col("output").struct["perf"].struct["branch"]
SELECT_BRANCH_MISS = pl.col("output").struct["perf"].struct["branch_miss"]


def display_abs(value) -> str:
    suffix = ""
    divisor = 1

    if value < 1e3:
        pass
    elif value < 1e6:
        suffix = "K"
        divisor = int(1e3)
    elif value < 1e9:
        suffix = "M"
        divisor = int(1e6)
    elif value < 1e12:
        suffix = "B"
        divisor = int(1e9)
    elif value < 1e15:
        suffix = "T"
        divisor = int(1e12)

    if isinstance(value, int):
        return f"{value // divisor}{suffix}"
    else:
        return f"{value / divisor:.1f}{suffix}"

    assert False


def display_rel(ratio: float) -> str:
    return f"{ratio:.2f}x"
