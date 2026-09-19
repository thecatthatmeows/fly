import io
import pandas as pd
from fafbseg import flywire

# Read your activity data
df_input = pd.read_csv("out_data/output_neurons.csv")

# Make sure FlyWire root IDs are treated as strings.
# These IDs are 64-bit and should not go through floating-point conversion.
df_input["neuron_id"] = df_input["neuron_id"].astype(str)

# ---------------------------------------------------------
# Load FlyWire's public annotation table
# ---------------------------------------------------------

annotation_url = (
    "https://raw.githubusercontent.com/flyconnectome/"
    "flywire_annotations/master/"
    "supplemental_files/Supplemental_file1_neuron_annotations.tsv"
)

print("Fetching public FlyWire annotations...")

annotations_df = pd.read_csv(
    annotation_url,
    sep="\t",
    dtype={"root_id": str}
)

# ---------------------------------------------------------
# Merge annotations with your activity data
# ---------------------------------------------------------

merged_df = pd.merge(
    df_input,
    annotations_df,
    left_on="neuron_id",
    right_on="root_id",
    how="left"
)

# ---------------------------------------------------------
# Count neurons by superclass
# ---------------------------------------------------------

merged_df["super_class"] = (
    merged_df["super_class"]
    .fillna("Unassigned")
)

super_class_summary = (
    merged_df["super_class"]
    .value_counts()
    .reset_index()
)

super_class_summary.columns = [
    "Super Class",
    "Neuron Count"
]

print("\n--- Neuron Counts by Super Class ---")
print(super_class_summary.to_string(index=False))

# ---------------------------------------------------------
# Top specific cell types
# ---------------------------------------------------------

if "cell_type" in merged_df.columns:

    cell_type_summary = (
        merged_df["cell_type"]
        .fillna("Unassigned")
        .value_counts()
        .head(15)
        .reset_index()
    )

    cell_type_summary.columns = [
        "Specific Cell Type",
        "Neuron Count"
    ]

    print("\n--- Top Specific Cell Types ---")
    print(cell_type_summary.to_string(index=False))

# ---------------------------------------------------------
# Optional: show activity statistics by superclass
# ---------------------------------------------------------

activity_summary = (
    merged_df
    .groupby("super_class", dropna=False)
    .agg(
        Neuron_Count=("neuron_id", "count"),
        Mean_Activity=("activity", "mean"),
        Total_Activity=("activity", "sum")
    )
    .sort_values("Neuron_Count", ascending=False)
)

print("\n--- Activity by Super Class ---")
print(activity_summary.to_string())