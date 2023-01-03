use crate::queue::{Task, WorkQueue};
use digest::consts::U32;
use sha2::digest::generic_array::GenericArray;
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::sync;

pub type Hash = GenericArray<u8, U32>;

#[derive(Debug, Clone)]
pub struct Block {
    pub prev_hash: Hash,
    pub generation: u64,
    pub difficulty: u8,
    pub data: String,
    pub proof: Option<u64>,
}

impl Block {
    // Given a difficulty d, the initial function will create a block with: a hash of all zeros, a generation
    // of 0, a difficulty of d, a data of "", and a proof of None. To create a hash of all zeroes, one can call
    // Hash::default()
    pub fn initial(difficulty: u8) -> Block {
        return Block {
            prev_hash: Hash::default(),
            generation: 0,
            difficulty,
            data: String::from(""),
            proof: None,
        };
    }

    // Create and return a block that could follow `previous` in the chain
    // create new block with the same difficulty as previous
    // should have the previous_hash set to the hash of the previous Block
    // the generation should be 1 higher than the last generation
    // the difficulty should remain the same
    // proof is None
    pub fn next(previous: &Block, data: String) -> Block {
        return Block {
            prev_hash: previous.hash(),
            generation: previous.generation + 1,
            difficulty: previous.difficulty,
            data,
            proof: None,
        };
    }

    // Return the hash string this block would have if we set the proof to `proof`.
    // The hash_string_for_proof function should create a string formatted as follows:
    // previous_hash : generation : difficulty : data : proof
    pub fn hash_string_for_proof(&self, proof: u64) -> String {
        let hash_string = format!("{:02x}", self.prev_hash);
        return format!(
            "{}:{}:{}:{}:{}",
            hash_string,
            self.generation,
            self.difficulty,
            self.data,
            proof.to_string()
        );
    }

    pub fn hash_string(&self) -> String {
        // self.proof.unwrap() panics if block not mined
        let p = self.proof.unwrap();
        self.hash_string_for_proof(p)
    }

    // Return the block's hash as it would be if we set the proof to `proof`.
    pub fn hash_for_proof(&self, proof: u64) -> Hash {
        let mut digest = Sha256::new();
        digest.update(self.hash_string_for_proof(proof));
        return digest.finalize();
    }

    pub fn hash(&self) -> Hash {
        // self.proof.unwrap() panics if block not mined
        let p = self.proof.unwrap();
        self.hash_for_proof(p)
    }

    pub fn set_proof(self: &mut Block, proof: u64) {
        self.proof = Some(proof);
    }

    // Does the hash `hash` have `difficulty` trailing 0s
    pub fn hash_satisfies_difficulty(difficulty: u8, hash: Hash) -> bool {
        // 1. Define n_bytes as the difficulty divided by 8
        // 2. Define n_bits as the difficulty mod 8
        // 3. Check that each of the last n_bytes are 0u8
        // 4. Check that the byte one before the last n_bytes is divisible by 1<<n_bits (as 1<<n_bits == 2
        // n_bits).

        let n_bytes = difficulty / 8;
        let n_bits = difficulty % 8;
        let last_n_bytes = hash.len() - (n_bytes as usize);

        for i in last_n_bytes..hash.len() {
            if hash[i] != 0u8 {
                return false;
            }
        }

        let mod_val = 1 << n_bits;

        if (hash[last_n_bytes - 1] % mod_val) != 0 {
            return false;
        }

        return true;
    }

    pub fn is_valid_for_proof(&self, proof: u64) -> bool {
        Self::hash_satisfies_difficulty(self.difficulty, self.hash_for_proof(proof))
    }

    pub fn is_valid(&self) -> bool {
        if self.proof.is_none() {
            return false;
        }
        self.is_valid_for_proof(self.proof.unwrap())
    }

    // Mine in a very simple way: check sequentially until a valid hash is found.
    // This doesn't *need* to be used in any way, but could be used to do some mining
    // before your .mine is complete. Results should be the same as .mine (but slower).
    pub fn mine_serial(self: &mut Block) {
        let mut p = 0u64;
        while !self.is_valid_for_proof(p) {
            p += 1;
        }
        self.proof = Some(p);
    }

    pub fn mine_range(self: &Block, workers: usize, start: u64, end: u64, chunks: u64) -> u64 {
        // TODO: with `workers` threads, check proof values in the given range, breaking up
        // into `chunks` tasks in a work queue. Return the first valid proof found.
        // HINTS:
        // - Create and use a queue::WorkQueue.
        // - Use sync::Arc to wrap a clone of self for sharing.
        let mut work_queue = WorkQueue::new(workers);
        let arc_block = sync::Arc::new(self.clone());
        let range = (end - start) / chunks;

        let mut start_point = 0;
        let mut end_point = range;

        for _ in 0..chunks {
            let task = MiningTask {
                block: arc_block.clone(),
                start: start_point,
                end: end_point,
            };
            work_queue.enqueue(task).unwrap();

            start_point = start_point + range;
            end_point = end_point + range;
        }

        let r = work_queue.recv();
        work_queue.shutdown();
        return r;
    }

    pub fn mine_for_proof(self: &Block, workers: usize) -> u64 {
        let range_start: u64 = 0;
        let range_end: u64 = 8 * (1 << self.difficulty); // 8 * 2^(bits that must be zero)
        let chunks: u64 = 2345;
        self.mine_range(workers, range_start, range_end, chunks)
    }

    pub fn mine(self: &mut Block, workers: usize) {
        self.proof = Some(self.mine_for_proof(workers));
    }
}

struct MiningTask {
    block: sync::Arc<Block>,
    start: u64,
    end: u64,
}

impl Task for MiningTask {
    type Output = u64;

    fn run(&self) -> Option<u64> {
        // Iterate through every number o the chunk and check whether that number
        // is a valid proof
        // If it is return some of that proof, if none are, return None

        for i in self.start..self.end {
            if self.block.is_valid_for_proof(i) {
                return Some(i);
            }
        }
        return None;
    }
}
