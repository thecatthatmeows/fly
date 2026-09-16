use std::{fs::{self, File}, io::{self, Write}};
use csv::{Reader, Result};
use rand::random_range;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rkyv::{rancor, vec::ArchivedVec};

/// A neuron connection
#[derive(Debug, Clone, serde::Deserialize, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct ConnectionData {
    pub pre_root_id: i64,
    pub post_root_id: i64,
    pub neuropil: String,
    pub syn_count: u64,
    pub nt_type: String
}

#[derive(Debug, Clone, serde::Deserialize, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct ConnectionState {
    pub weight: f32
}

impl ConnectionState {
    pub fn new(weight: f32) -> Self {
        Self {
            weight
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct Connection {
    pub data: ConnectionData,
    pub state: ConnectionState
}

impl Connection {
    pub fn new(data: ConnectionData) -> Self {
        Self {
            data,
            state: ConnectionState::new(1.0)
        }
    }

    pub fn nt_sign(&self) -> f32 {
        match self.data.nt_type.as_str() {
            "ACH"  =>  1.0,
            "GABA" => -1.0,
            "GLUT" => -1.0,
            "DA"   =>  1.0,
            "5HT"  =>  1.0,
            "OCT"  =>  1.0,
            _      =>  1.0, // temporary fallback
        }
    }
}

#[derive(Clone)]
pub struct Connections {
    pub conns: Vec<Connection>
}

impl Connections {
    pub fn new() -> Result<Self> {
        let mut conns = Vec::new();
        let bytes = fs::read("connections.rkyv");

        if let Ok(bytes) = bytes {
            println!(".rkyv file cache found! Using that cache.");
            let archived = 
                rkyv::access::<ArchivedVec<ArchivedConnection>, rancor::Error>(&bytes).unwrap();
            println!("Connections has been accessed from the .rkyv cache file, length: {}", archived.len());
            conns = archived.par_iter().map(|archived_conn| {
                rkyv::deserialize::<Connection, rancor::Error>(archived_conn).unwrap()
            }).collect();
            println!("Connections have been deserialized from the .rkyv cache file, length: {}", conns.len());

            return Ok(Connections { conns });
        }

        // first pass count rows
        let file = File::open("connections.csv")?;
        let mut reader = Reader::from_reader(file);
        let row_count = reader.records().count();

        // second pass actually read the data
        let file = File::open("connections.csv")?;
        let mut csv_reader = Reader::from_reader(file);

        conns = csv_reader
            .deserialize()
            .enumerate()
            .par_bridge()
            .map(|(i, res)| {
                let progress = ((i+1) as f64 / row_count as f64) * 100.0;
                print!("\rLoading connections... [{:.2}%]", progress);
                io::stdout().flush().unwrap();

                let record: ConnectionData = res.unwrap();
                Connection::new(record)
            }).collect::<Vec<Connection>>(); // Force evaluation of the parallel iterator
        let bytes = rkyv::to_bytes::<rancor::Error>(&conns).unwrap();
        fs::write("connections.rkyv", bytes)?;
        println!("connections cached, using that the next time this program runs");

        Ok(Connections { conns })
    }
}