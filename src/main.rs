use std::{fs::File, io::{self, BufRead, BufReader, Write}, sync::atomic::{AtomicUsize, Ordering}};

use crate::{brain::Brain, connection::{Connection, Connections}, fly::Fly};
use csv::Result;
use raylib::prelude::*;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

mod connection;
mod brain;
mod fly;

fn _get_neuron_with_least_connections(conns: &[Connection], brain: &Brain) {
    // find the one with the least connections
    let mut completed = AtomicUsize::new(0);
    let total = conns.len();
    let (min_neuron_id, min_len) =
    conns.par_iter().enumerate().map(|(i, conn)| {
        let neuron_id = conn.data.pre_root_id;
        let affecteds = brain.hop_propagate(neuron_id);

        let len = affecteds.len();
        
        let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
        if done % 1000 == 0 || done == total {
            let progress = (done as f64 / total as f64) * 100.0;
            print!("\rFinding minimum connections... [{:.2}%]", progress);
            io::stdout().flush().unwrap();
        }

        (neuron_id, len)
    })
    .min_by_key(|&(_, len)| len)
    .unwrap_or((0, usize::MAX));

    println!("The minimum number of connections is {}", min_len);
    println!("The neuron with the minimum connections is {}", min_neuron_id);
}

fn main() -> Result<()> {
    let conns = Connections::new()?;

    let mut starting_neuron_ids = Vec::new();

    let ids_limit = 0;
    let ids_file = File::open("neuron_ids/gustatory/ids.txt")?;
    let ids_reader = BufReader::new(ids_file);
    for (i, id_line) in ids_reader.lines().enumerate() {
        if i >= ids_limit && (ids_limit != 0) {
            break;
        }
        let id = id_line?.parse::<i64>().unwrap();
        starting_neuron_ids.push(id);
    }

    let mut brain = Brain::new(
        starting_neuron_ids.clone(),
        Vec::new(),
        Some(&conns),
        0.01
    )?; 

    let limit = 0;
    let epochs = 100;

    let mut propagated_neurons = Vec::new();
    let mut learnt_neurons = Vec::new();

    for i in 0..epochs {
        println!("Epoch {}/{}  ", i+1, epochs);
        brain.reset_activities();
        brain.stimulate_many(&starting_neuron_ids, 1.0);

        propagated_neurons = brain.propagate(&starting_neuron_ids, limit);
        learnt_neurons = brain.learn(&starting_neuron_ids, limit);
    }

    let output = brain.get_output();
    for (id, activity) in &output {
        println!("Neuron ID: {id} Activity: {activity}");
    }
    println!("There are {} output neurons", output.len());

    let mut writer_propagated = csv::Writer::from_path("out_data/propagated.csv")?;
    writer_propagated.write_record(&["pre_root_id", "post_root_id", "activity"])?;
    for (pre, post, activity) in &propagated_neurons {
        writer_propagated.write_record(&[pre.to_string(), post.to_string(), activity.to_string()])?;
    }
    writer_propagated.flush()?;
    println!("Saved propagated neurons");

    let mut writer_learnt = csv::Writer::from_path("out_data/learnt.csv")?;
    writer_learnt.write_record(&["pre_root_id", "post_root_id", "weight"])?;
    for (pre, post, weight, _) in &learnt_neurons {
        writer_learnt.write_record(&[pre.to_string(), post.to_string(), weight.to_string()])?;
    }
    writer_learnt.flush()?;
    println!("Saved learnt neurons");

    // let (mut rl, thread) = raylib::init()
    //     .size(600, 600)
    //     .title("meow")
    //     .build();

    // let mut fly = Fly::new(300.0, 300.0);
    // while !rl.window_should_close() {
    //     let mut d = rl.begin_drawing(&thread);
    //     d.clear_background(Color::BLACK);
        
    //     fly.draw(&mut d);
    //     fly.update();
    // }

    Ok(())
}
