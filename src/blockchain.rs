use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::LazyLock,
    time::{Duration, SystemTime},
};

// How often should a block be find
const BLOCK_GENERATION_INTERVAL: Duration = Duration::new(4, 0);
// How often difficulty should be adjusted
const DIFFICULTY_ADJUSTMENT_INTERVAL: usize = 10;

// Ensures every node have the same first block
static GENESIS_BLOCK: LazyLock<Block> = LazyLock::new(|| {
    let data = BlockData {
        index: 0,
        previous_hash: 0,
        timestamp: SystemTime::now(),
        message: "Hello, world!".into(),
        nonce: 0,
        // How many high bits of the hash need to be 0
        // difficulty varys along the generation of new blocks
        difficulty: 8,
    };
    Block::from_data(data)
});

#[derive(Hash, Clone)]
struct BlockData {
    // Fields needed for a block chain
    index: usize,
    previous_hash: u64,
    timestamp: SystemTime,
    // Additional fields
    message: String,
    nonce: u64,
    difficulty: u32,
}

#[derive(Clone)]
pub struct Block {
    data: BlockData,
    hash: u64,
}

impl Block {
    fn from_data(data: BlockData) -> Self {
        let mut data = data;
        let hash = loop {
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            let hash = hasher.finish();
            if hash >> (u64::BITS - data.difficulty) == 0 {
                break hash;
            }
            data.nonce += 1;
        };

        Block { data, hash }
    }

    fn validate_with_prev(&self, prev: &Block) -> bool {
        let mut hasher = DefaultHasher::new();
        self.data.hash(&mut hasher);
        let hash = hasher.finish();

        prev.data.index + 1 == self.data.index
            && prev.hash == self.data.previous_hash
            && self.hash == hash
            && hash >> (u64::BITS - self.data.difficulty) == 0
            && prev.data.timestamp
                < self
                    .data
                    .timestamp
                    .checked_add(Duration::new(60, 0))
                    .unwrap()
    }

    fn validate_with_prev_realtime(&self, prev: &Block) -> bool {
        self.validate_with_prev(prev)
            && self.data.timestamp < SystemTime::now().checked_add(Duration::new(60, 0)).unwrap()
    }
}

#[derive(Clone)]
pub struct Chain {
    local_queue: Vec<Block>,
    cumulative_diffculty: usize,
}

impl Default for Chain {
    fn default() -> Self {
        Chain {
            local_queue: vec![GENESIS_BLOCK.clone()],
            cumulative_diffculty: 0,
        }
    }
}

impl Chain {
    pub fn get_latest(&self) -> &Block {
        self.local_queue.last().unwrap()
    }

    fn add_block_helper(self: &mut Self, block: Block) {
        self.cumulative_diffculty += 1 << block.data.difficulty;
        self.local_queue.push(block);
    }

    pub fn generate_block(self: &mut Self, message: String) -> Block {
        let last_block = self.local_queue.last().unwrap();
        let index = last_block.data.index + 1;
        let previous_hash = last_block.hash;
        let timestamp = SystemTime::now();
        let new_data = BlockData {
            index,
            previous_hash,
            timestamp,
            message,
            nonce: 0,
            difficulty: self.get_difficulty(),
        };
        let new_block = Block::from_data(new_data);
        self.add_block_helper(new_block.clone());
        new_block
    }

    fn adjust_difficulty(&self, blk: &Block) -> u32 {
        let prev_adjust_block =
            self.local_queue[self.local_queue.len() - DIFFICULTY_ADJUSTMENT_INTERVAL].clone();
        let time_taken = blk
            .data
            .timestamp
            .duration_since(prev_adjust_block.data.timestamp)
            .unwrap();
        let time_expected =
            BLOCK_GENERATION_INTERVAL.mul_f64(DIFFICULTY_ADJUSTMENT_INTERVAL as f64);

        let threshold = 0.3;
        if time_taken < time_expected.mul_f64(1.0 - threshold) {
            prev_adjust_block.data.difficulty + 1
        } else if time_taken > time_expected.mul_f64(1.0 + threshold) {
            prev_adjust_block.data.difficulty - 1
        } else {
            prev_adjust_block.data.difficulty
        }
    }

    fn get_difficulty(&self) -> u32 {
        let latest_block = self.get_latest();

        if latest_block.data.index % DIFFICULTY_ADJUSTMENT_INTERVAL == 0
            && latest_block.data.index != 0
        {
            return self.adjust_difficulty(latest_block);
        } else {
            return latest_block.data.difficulty;
        }
    }

    fn validate(&self) -> bool {
        for window in self.local_queue.as_slice().windows(2) {
            if !window[1].validate_with_prev(&window[0]) {
                return false;
            }
        }

        true
    }

    pub fn try_replace_with(&mut self, other: Chain) {
        if other.validate() && self.cumulative_diffculty < other.cumulative_diffculty {
            *self = other;
        }
    }

    pub fn try_append(&mut self, target: Block) -> bool {
        let last = self.get_latest();
        if target.validate_with_prev_realtime(last) {
            self.add_block_helper(target);
            true
        } else {
            false
        }
    }
}
