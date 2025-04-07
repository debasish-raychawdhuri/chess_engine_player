use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    path::Path,
};

use crate::error::AppError;

pub struct ChessEngine {
    process: Option<Child>,
    move_receiver: mpsc::Receiver<String>,
    move_sender: mpsc::Sender<String>,
    think_time: u64,
}

impl ChessEngine {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        
        ChessEngine {
            process: None,
            move_receiver: rx,
            move_sender: tx,
            think_time: 2000,
        }
    }
    
    pub fn start<P: AsRef<Path>>(&mut self, engine_path: P, skill_level: u8, think_time: u64) -> Result<(), AppError> {
        let path = engine_path.as_ref();
        if !path.exists() {
            return Err(AppError::Engine(format!("Engine not found at path: {:?}", path)));
        }

        let process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        self.process = Some(process);
        self.think_time = think_time;
        
        // Initialize UCI engine
        if let Some(ref mut process) = self.process {
            let mut stdin = process.stdin.take().unwrap();
            stdin.write_all(b"uci\n")?;
            stdin.write_all(b"isready\n")?;
            
            // Set engine options
            stdin.write_all(format!("setoption name Skill Level value {}\n", skill_level).as_bytes())?;
            stdin.write_all(b"setoption name Threads value 4\n")?;
            stdin.write_all(b"setoption name Hash value 128\n")?;
            stdin.write_all(b"setoption name UCI_AnalyseMode value false\n")?;
            stdin.write_all(b"setoption name UCI_LimitStrength value false\n")?;
            stdin.flush()?;
            
            // Read engine output in a separate thread
            let stdout = process.stdout.take().unwrap();
            let reader = BufReader::new(stdout);
            
            // Get a clone of the sender to pass to the thread
            let tx_clone = self.move_sender.clone();
            
            thread::spawn(move || {
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if line.starts_with("bestmove") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                tx_clone.send(parts[1].to_string()).unwrap_or(());
                            }
                        }
                    }
                }
            });
            
            // Return stdin to the process
            process.stdin = Some(stdin);
        }
        
        Ok(())
    }
    
    pub fn get_move(&mut self, fen: &str) -> Result<(), AppError> {
        if let Some(ref mut process) = self.process {
            if let Some(stdin) = process.stdin.as_mut() {
                // Send position to engine
                let position_cmd = format!("position fen {}\n", fen);
                stdin.write_all(position_cmd.as_bytes())?;
                
                // Ask engine to think
                let think_cmd = format!("go movetime {}\n", self.think_time);
                stdin.write_all(think_cmd.as_bytes())?;
                stdin.flush()?;
                return Ok(());
            }
        }
        
        Err(AppError::Engine("Engine process not available".to_string()))
    }
    
    pub fn try_receive_move(&self) -> Option<String> {
        match self.move_receiver.try_recv() {
            Ok(best_move) => Some(best_move),
            Err(_) => None,
        }
    }
    
    pub fn set_think_time(&mut self, think_time: u64) {
        self.think_time = think_time;
    }
}

impl Drop for ChessEngine {
    fn drop(&mut self) {
        if let Some(ref mut process) = self.process {
            if let Some(stdin) = process.stdin.as_mut() {
                let _ = stdin.write_all(b"quit\n");
                let _ = stdin.flush();
            }
            let _ = process.kill();
        }
    }
}
