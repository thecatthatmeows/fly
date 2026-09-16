import matplotlib.pyplot as plt
import networkx as nx
import csv
import matplotlib
matplotlib.use("qtagg")

limit = int(input("Enter the limit of nodes (0 if you dont care about your performance): "))

with open("out_data/propagated.csv", "r") as csvfile:
    reader = csv.reader(csvfile)
    next(reader)  # Skip header

    rows = list(reader)

fig_manager = plt.get_current_fig_manager()
fig_manager.full_screen_toggle()

G = nx.DiGraph()
for (i, (pre, post, activity)) in enumerate(rows):
    activity = round(float(activity), 2)
    G.add_edge(pre, post, label=activity)
    if i >= limit and (limit != 0):
        break

pos = nx.kamada_kawai_layout(G)
high_edges = [
    (u, v) for u, v, data in G.edges(data=True) if data["label"] >= 0.5
]
low_edges = [
    (u, v) for u, v, data in G.edges(data=True) if data["label"] < 0.5
]

nx.draw_networkx_nodes(G, pos, node_color="lightblue", node_size=25)
nx.draw_networkx_edges(G, pos, edgelist=high_edges, edge_color="lime")
nx.draw_networkx_edges(G, pos, edgelist=low_edges, edge_color="red")
nx.draw_networkx_edge_labels(G, pos, edge_labels=nx.get_edge_attributes(G, "label"))
plt.title("Directed Graph Visualization")
plt.show()