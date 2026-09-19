use std::collections::HashSet;
use std::{fs, io};
use std::{collections::HashMap, io::Write};
use std::sync::Mutex;
use csv::Result;
use rand::random_range;
use rayon::iter::IntoParallelRefIterator;
use crate::connection::{Connection, Connections};

#[derive(serde::Deserialize)]
struct Neuron {
    /// Uhh identifier?
    root_id: i64,

    /// How likely it is to fire
    activity: f32,

    #[serde(default)]
    /// Activity threshold
    activity_threshold: f32,

    // Activity threshold baseline
    // activity_threshold_base: f32,
    cell_class: String,
}

impl Neuron {
    pub fn new(root_id: i64) -> Self {
        Self {
            root_id,
            activity: 0.0,
            activity_threshold: 0.001,
            cell_class: String::new(),
            // activity_threshold_base: 0.5,
        }
    }
}

pub struct Brain {
    /// All neurons in the simulated brain, indexed by their root ID.
    neurons: HashMap<i64, Mutex<Neuron>>,
    /// Maps each neuron to the neurons it is connected to.
    coupling: HashMap<i64, Vec<Connection>>,
    learning_rate: f32,
    weight_decay: f32,
    activity_keep: f32,
    /// Neuron IDs which are used as inputs at the moment
    current_inputs: Vec<i64>,
    /// Neuron IDs which are used as outputs at the moment
    current_outputs: Vec<i64>
}

impl Brain {
    pub fn new(
        neuron_input_ids: Vec<i64>,
        neuron_output_ids: Vec<i64>,
        connections: Option<&Connections>,
        learning_rate: f32,
    ) -> Result<Self> {
        println!("Initializing...");
        let conns = if let Some(conns) = connections {
            println!("Using {} provided connections", conns.conns.len());
            conns.clone()
        } else {
            println!("Loading connections...");
            let conns = Connections::new()?;
            println!("Loaded {} connections", conns.conns.len());
            conns
        };

        println!("Initializing coupling connections...");

        let mut coupling_connections: HashMap<i64, Vec<Connection>> =
            HashMap::new();

        for conn in &conns.conns {
            coupling_connections
                .entry(conn.data.pre_root_id)
                .or_default()
                .push(conn.clone());
        }

        println!(
            "Coupling connections initialized: {} pre-neurons",
            coupling_connections.len()
        );

        println!("Initializing neuron nodes...");

        let mut neurons: HashMap<i64, Neuron> = HashMap::new();

        for conn in &conns.conns {
            neurons
                .entry(conn.data.pre_root_id)
                .or_insert(Neuron::new(conn.data.pre_root_id));

            neurons
                .entry(conn.data.post_root_id)
                .or_insert(Neuron::new(conn.data.post_root_id));
        }

        // Ensure all declared input neurons exist
        for neuron_id in &neuron_input_ids {
            neurons
                .entry(*neuron_id)
                .or_insert(Neuron::new(*neuron_id));
        }

        println!("Neuron nodes initialized: {}", neurons.len());
        println!("Wrapping neurons...");

        let neurons = neurons
            .into_iter()
            .map(|(id, neuron)| (id, Mutex::new(neuron)))
            .collect();

        println!("Initialization complete.");

        Ok(Self {
            coupling: coupling_connections,
            neurons,
            learning_rate,
            weight_decay: 0.0001,
            activity_keep: 0.9,
            current_inputs: neuron_input_ids,
            current_outputs: neuron_output_ids,
        })
    }

    pub fn reset_activities(&self) {
        for neuron in self.neurons.values() {
            let mut neuron = neuron.lock().unwrap();
            neuron.activity = 0.0;
        }
    }

    pub fn stimulate(&self, neuron_id: i64, activity: f32) {
        let mut neuron = 
            self.neurons.get(&neuron_id).unwrap().lock().unwrap();
        neuron.activity = activity;
    }

    pub fn stimulate_many(&self, neuron_ids: &[i64], activity: f32) {
        for neuron_id in neuron_ids {
            self.stimulate(*neuron_id, activity);
        }
    }

    /// Sets the current brain's input to given neuron_input_ids
    /// and later sets the input neuron's activity to given activity, if there is
    pub fn set_input(&mut self, neuron_input_ids: Vec<i64>, activity: Option<f32>) {
        self.current_inputs = neuron_input_ids;
        for neuron_id in &self.current_inputs {
            let mut neuron =
                self.neurons.get(neuron_id).unwrap().lock().unwrap();

            if let Some(activity) = activity {
                neuron.activity = activity;
            }
        }
    }

    /// returns (neuron_id, activity)
    pub fn get_output(&self) -> Vec<(i64, f32)> {
        self.current_outputs
            .iter()
            .filter_map(|id| {
                self.neurons.get(id).map(|neuron| {
                    let neuron = neuron.lock().unwrap();
                    (*id, neuron.activity)
                })
            })
            .collect()
    }

    pub fn search_neurons(
        &self,
        starting_neuron_ids: &[i64],
        limit: usize,
    ) -> Vec<i64> {
        let mut root_ids = starting_neuron_ids.to_vec();
        let mut idx = 0;

        let mut queued: HashSet<i64> =
            starting_neuron_ids.iter().copied().collect();

        while idx < root_ids.len()
            && (limit == 0 || idx < limit)
        {
            let root_id = root_ids[idx];

            let (activity, threshold) = {
                let neuron = self.neurons
                    .get(&root_id)
                    .unwrap()
                    .lock()
                    .unwrap();

                (neuron.activity, neuron.activity_threshold)
            };

            if activity.abs() >= threshold {
                if let Some(conns) = self.coupling.get(&root_id) {
                    for conn in conns {
                        let post_root_id = conn.data.post_root_id;

                        if queued.insert(post_root_id) {
                            root_ids.push(post_root_id);
                        }
                    }
                }
            }

            idx += 1;
        }

        root_ids
    }

    /// returns activity through the network of neurons, starting from the given neuron_id
    /// which returns a list of tuples of (pre_root_id, post_root_id, new_activity)
    pub fn hop_propagate(&self, neuron_id: i64) -> Vec<(i64, i64, f32)> {
        let mut affecteds = Vec::new();

        if let Some(conns) = self.coupling.get(&neuron_id) {
            let pre_activity = self.neurons.get(&neuron_id).unwrap().lock().unwrap().activity;
            for conn in conns {
                // WARNING: There might be a missing post neuron, dont blame me aight?
                // do something with an individual neuron
                let post_neuron = self.neurons.get(&conn.data.post_root_id).unwrap().lock().unwrap();
                let signal = pre_activity * conn.state.weight * conn.nt_sign();
                let new_activity = post_neuron.activity + signal;

                affecteds.push((conn.data.pre_root_id, conn.data.post_root_id, new_activity));
            }
        }

        affecteds
    }
    
    /// returns (pre_root_id, post_root_id, activity)
    pub fn propagate(&mut self, starting_neuron_ids: &[i64], limit: usize) -> Vec<(i64, i64, f32)> {
        let mut affecteds = Vec::new();
        let root_ids = self.search_neurons(starting_neuron_ids, limit);

        self.current_outputs = root_ids
            .iter()
            .copied()
            .filter(|id| {
                match self.coupling.get(id) {
                    Some(conns) => {
                        conns
                            .iter()
                            .any(|conn| {
                                if let Some(conns) = self.coupling.get(&conn.data.post_root_id) {
                                    !conns.iter().any(|conn| {
                                        root_ids.contains(&conn.data.post_root_id)
                                    })
                                } else {
                                    true
                                }
                            })
                    }
                    None => true
                }
            })
            .collect();

        for root_id in root_ids {
            // fires
            let outputs = self.hop_propagate(root_id);
            affecteds.extend(outputs);
        }

        let mut inputs = HashMap::new();
        for (_, post_root_id, activity) in &affecteds {
            *inputs.entry(post_root_id).or_insert(0.0) += activity;
        }

        for (post_root_id, activity) in inputs {
            let mut post_neuron = self.neurons.get(&post_root_id).unwrap().lock().unwrap();
            post_neuron.activity = (activity * self.activity_keep).clamp(-1.0, 1.0);
        }

        println!("\nPropagation complete. Total affected connections: {}", affecteds.len());

        affecteds
    }

    /// returns weights of connections based on the activity of the neurons
    /// decides whether the connection strength should be stronger or weaker
    pub fn hop_learn(&mut self, neuron_id: i64) -> Vec<(i64, i64, f32)> {
        let mut affecteds = Vec::new();

        if let Some(conns) = self.coupling.get_mut(&neuron_id) {
            let (pre_activity, pre_root_id) = {
                let neuron = self.neurons.get(&neuron_id).unwrap().lock().unwrap();
                (neuron.activity, neuron.root_id) 
            };
            for conn in conns.iter_mut() {
                let (post_activity, post_root_id) = {
                    let neuron =
                        self.neurons.get(&conn.data.post_root_id).unwrap().lock().unwrap();
                    (neuron.activity, neuron.root_id) 
                };

                // Hebbian learning
                let delta_weight =
                    self.learning_rate * (pre_activity * post_activity - self.weight_decay);
                let new_weight = 
                    (conn.state.weight + delta_weight).clamp(0.0, 1.0);

                affecteds.push((pre_root_id, post_root_id, new_weight));
            }
        }
        affecteds
    }

    /// returns (pre_root_id, post_root_id, weight, activity)
    pub fn learn(&mut self, starting_neuron_ids: &[i64], limit: usize) -> Vec<(i64, i64, f32, f32)> {
        let mut affecteds = Vec::new();

        let root_ids = self.search_neurons(starting_neuron_ids, limit);
        for root_id in root_ids {
            let activity = 
                self.neurons.get(&root_id).unwrap().lock().unwrap().activity;
            let outputs = self.hop_learn(root_id);

            for (pre_root_id, post_root_id, weight) in outputs {
                affecteds.push((
                    pre_root_id,
                    post_root_id,
                    weight,
                    activity,
                ));
            }
        }
        println!("\nLearning complete. Total affected neurons: {}", affecteds.len());

        affecteds
    }

    fn get_neurons_by_class(&self, cell_class: &str) -> Result<Vec<Neuron>> {
        let mut res_neurons = Vec::new();

        let mut csv_reader = 
            csv::Reader::from_path("out_data/output_neurons_classified.csv")?;

        let mut neurons = Vec::new();
        for res in csv_reader.deserialize() {
            let neuron: Neuron = res?;
            neurons.push(neuron);
        }

        for neuron in neurons {
            if neuron.cell_class == cell_class {
                res_neurons.push(neuron);
            }
        }

        Ok(res_neurons)
    }
}