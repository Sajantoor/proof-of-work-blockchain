#[cfg(test)]
mod block_tests {
    use crate::block::*;

    #[test]
    fn initial_basic_0() {
        let b0: Block = Block::initial(13);
        assert_eq!(b0.difficulty, 13);
        assert_eq!(b0.generation, 0);
        assert_eq!(b0.prev_hash, Hash::from([0; 32]));
        assert_eq!(b0.data, "");
        assert_eq!(b0.proof, None);
    }

    #[test]
    fn hash_string_for_proof_basic_0() {
        let b0: Block = Block {
            difficulty: 13,
            generation: 3,
            prev_hash: Hash::from([10; 32]),
            data: "Cool Data".to_string(),
            proof: Option::None,
        };
        assert_eq!("0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a:3:13:Cool Data:4321"
                  ,b0.hash_string_for_proof(4321))
    }

    #[test]
    fn hash_for_proof_basic_0() {
        let b0: Block = Block {
            difficulty: 13,
            generation: 3,
            prev_hash: Hash::from([10; 32]),
            data: "Cool Data".to_string(),
            proof: Option::None,
        };
        assert_eq!(Hash::from([
                        99, 66, 200, 198, 96, 57, 238, 158, 136, 127, 33, 80, 24, 122, 108, 205,
                        44, 40, 7, 58, 131, 224, 179, 144, 96, 228, 207, 83, 74, 179, 142, 115
                        ])
                  ,b0.hash_for_proof(4321))
    }

    #[test]
    fn next_basic_0() {
        let b0: Block = Block {
            difficulty: 13,
            generation: 3,
            prev_hash: Hash::from([10; 32]),
            data: "Cool Data".to_string(),
            proof: Option::Some(102020),
        };
        let b1 : Block = Block::next(&b0,"Cooler data".to_string());
        assert_eq!(b1.difficulty, 13);
        assert_eq!(b1.generation, 4);
        assert_eq!(b1.prev_hash, b0.hash());
        assert_eq!(b1.data, "Cooler data");
        assert_eq!(b1.proof, None);
    }

    #[test]
    fn hash_satisfies_difficulty_0() {
        assert!(Block::hash_satisfies_difficulty(8,Hash::from([
                        99, 66, 200, 198, 96, 57, 238, 158, 136, 127, 33, 80, 24, 122, 108, 205,
                        44, 40, 7, 58, 131, 224, 179, 144, 96, 228, 207, 83, 74, 179, 142, 0 
                        ])))
    }

    #[test]
    fn hash_satisfies_difficulty_1() {
        assert!(Block::hash_satisfies_difficulty(9,Hash::from([
                        99, 66, 200, 198, 96, 57, 238, 158, 136, 127, 33, 80, 24, 122, 108, 205,
                        44, 40, 7, 58, 131, 224, 179, 144, 96, 228, 207, 83, 74, 179, 142, 0 
                        ])))
    }

    #[test]
    fn hash_satisfies_difficulty_2() {
        assert!(!Block::hash_satisfies_difficulty(10,Hash::from([
                        99, 66, 200, 198, 96, 57, 238, 158, 136, 127, 33, 80, 24, 122, 108, 205,
                        44, 40, 7, 58, 131, 224, 179, 144, 96, 228, 207, 83, 74, 179, 142, 0 
                        ])))
    }

    #[test]
    fn mine_basic_0() {
        let mut b0: Block = Block {
            difficulty: 13,
            generation: 3,
            prev_hash: Hash::from([10; 32]),
            data: "Cool Data".to_string(),
            proof: Option::Some(102020),
        };
        b0.mine(4);
        assert!(b0.is_valid());
    }
}
