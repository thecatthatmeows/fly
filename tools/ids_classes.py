import pandas as pd

# Input activity CSV
df = pd.read_csv("out_data/output_neurons.csv")
df["neuron_id"] = df["neuron_id"].astype(str)

# Load FlyWire annotations
annotation_url = (
    "https://raw.githubusercontent.com/flyconnectome/"
    "flywire_annotations/master/"
    "supplemental_files/Supplemental_file1_neuron_annotations.tsv"
)

annotations = pd.read_csv(
    annotation_url,
    sep="\t",
    dtype={"root_id": str}
)

# Add cell class
class_map = annotations.set_index("root_id")["cell_class"]
df["cell_class"] = df["neuron_id"].map(class_map).fillna("Unknown")

# Save
df.to_csv("out_data/output_neurons_classified.csv", index=False)
print("Done.")