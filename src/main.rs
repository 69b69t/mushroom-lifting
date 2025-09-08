
//forward rng



fn mc_step_seed(seed: u64, salt: u64) -> u64 {
    seed
    .wrapping_mul(6364136223846793005u64)
    .wrapping_add(1442695040888963407u64)
    .wrapping_mul(seed)
    .wrapping_add(salt)
}

fn get_chunk_seed(start_seed: u64, x: i32, z: i32) -> u64 {
    let mut chunk_seed: u64 = start_seed.wrapping_add(x as u64);
    chunk_seed = mc_step_seed(chunk_seed, z as u64);
    chunk_seed = mc_step_seed(chunk_seed, x as u64);
    chunk_seed = mc_step_seed(chunk_seed, z as u64);
    chunk_seed
}

fn get_start_salt(world_seed: u64, layer_salt: u64) -> u64 {
    let mut start_salt = world_seed;
    start_salt = mc_step_seed(start_salt, layer_salt);
    start_salt = mc_step_seed(start_salt, layer_salt);
    start_salt = mc_step_seed(start_salt, layer_salt);
    start_salt
}

fn get_start_seed(world_seed: u64) -> u64 {
    let mut start_seed = world_seed;
    start_seed = get_start_salt(start_seed, 10967462438749070293u64);
    start_seed = mc_step_seed(start_seed, 0);
    start_seed
}

//---reverse---

fn gen_first_zero(i: u64) -> u64 {
    let mut seed: u64 = 100 * i;
    seed = seed << 24;
    seed
}

fn gen_proto_mush(i: u64, x: i32, z: i32) -> Vec<u64> {
    //this generates a seed that has a valid proto_mush at x z.
    let chunk_seed: u64 = gen_first_zero(i); //the bottom 24 bits of this start as zero, but can be anything
    let start_seeds = reverse_chunk_seed(chunk_seed, x, z);
    let world_seeds = reverse_start_seed_vec(start_seeds);
    world_seeds
}

//reverse chunk seeds

fn reverse_chunk_seed(chunk_seed: u64, x: i32, z: i32) -> Vec<u64> {
    //given a chunk_seed and a set of coords, find every start_seed that produced it.

    let mut start_seeds = lift_full(chunk_seed, z as u64);
    start_seeds = lift_vec(start_seeds, x as u64);
    start_seeds = lift_vec(start_seeds, z as u64);

    //perform wrapping subtraction by x on each element in the vector
    start_seeds.iter().map(|&seed| seed.wrapping_sub(x as u64)).collect()
}

fn reverse_chunk_seed_vec(chunk_seeds: Vec<u64>, x: i32, z: i32) -> Vec<u64> {
    let mut start_seeds: Vec<u64> = Vec::new();
    for chunk_seed in chunk_seeds {
        start_seeds.extend(reverse_chunk_seed(chunk_seed, x, z));
    }
    start_seeds.sort();
    start_seeds.dedup();
    start_seeds
}

//reverse start salts

fn reverse_start_salt(start_salt: u64, layer_salt: u64) -> Vec<u64> {
    //given a start_salt and a layer_salt, find every world_seed that produced it.
    let mut world_seeds = lift_full(start_salt, layer_salt);
    world_seeds = lift_vec(world_seeds, layer_salt);
    world_seeds = lift_vec(world_seeds, layer_salt);
    world_seeds
}

fn reverse_start_salt_vec(start_salts: Vec<u64>, layer_salt: u64) -> Vec<u64> {
    let mut world_seeds: Vec<u64> = Vec::new();
    for start_salt in start_salts {
        world_seeds.extend(reverse_start_salt(start_salt, layer_salt));
    }
    world_seeds.sort();
    world_seeds.dedup();
    world_seeds
}

//reverse start seed

fn reverse_start_seed(start_seed: u64) -> Vec<u64> {
    let mut world_seeds = lift_full(start_seed, 0);
    world_seeds = reverse_start_salt_vec(world_seeds, 10967462438749070293u64);
    world_seeds
}

fn reverse_start_seed_vec(start_seeds: Vec<u64>) -> Vec<u64> {
    let mut world_seeds: Vec<u64> = Vec::new();
    for start_seed in start_seeds {
        world_seeds.extend(reverse_start_seed(start_seed));
    }
    world_seeds.sort();
    world_seeds.dedup();
    world_seeds
}

//hensel lifting(reverse mc_step_seed)

fn lift_single(roots: Vec<u64>, k: u32, output_seed: u64, salt: u64) -> Vec<u64> {

    //lifts all roots to k+1 bits and return a vec of u64
    let mut next_roots: Vec<u64> = Vec::new();

    //let lifted_mask: u64 = (1 << (k + 1)) - 1;


    let lifted_mask: u64 = match k {
        0..=62 => (1 << (k + 1)) - 1,
        63 => 18446744073709551615, //2^64 - 1
        _ => 0
    };

    for root in roots {
        let result1 = mc_step_seed(root, salt);
        let result2 = mc_step_seed(root.wrapping_add(1 << k), salt);
        if result1 & lifted_mask == output_seed & lifted_mask {
            //println!("result1 was chosen because {} == {}", result1 & lifted_mask, output_seed & lifted_mask);
            next_roots.push(root);
        }
        if result2 & lifted_mask == output_seed & lifted_mask {
            //println!("result2 was chosen because {} == {}", result2 & lifted_mask, output_seed & lifted_mask);
            next_roots.push(root.wrapping_add(1 << k));
        }
        //println!("k={}, root={}, result1={}, result2={}, target={}", k, root, result1 & lifted_mask, result2 & lifted_mask, output_seed & lifted_mask);
    }
    next_roots
}

fn lift_full(output_seed: u64, salt: u64) -> Vec<u64> {
    let mut roots: Vec<u64> = Vec::new();
    roots.push(0);
    roots.push(1);
    for i in 0..64 {
        roots = lift_single(roots, i, output_seed, salt);
    }
    roots.sort();
    roots.dedup();
    roots
}

fn lift_vec(output_seeds: Vec<u64>, salt: u64) -> Vec<u64> {
    let mut roots: Vec<u64> = Vec::new();
    for seed in output_seeds {
        roots.extend(lift_full(seed, salt));
    }
    roots.sort();
    roots.dedup();
    roots
}

fn mc_first_is_zero(seed: u64) -> bool {
    ((seed as i64) >> 24) % 100 == 0
}

fn is_proto_mush(world_seed: u64, x: i32, z: i32) -> bool {
    let start_seed: u64 = get_start_seed(world_seed);

    //the upper bits dont affect the lower 26 bits, so as long as we
    //find a seed that matches all the criteria of the lower 26 bits,
    //we can pretty easily crack the upper few bits
    let chunk_seed: u64 = get_chunk_seed(start_seed, x, z);
    mc_first_is_zero(chunk_seed)
}

fn are_lower_bits_valid(positions: &[(i32, i32)], start_seed: u64) -> bool {
    for position in positions {
        let (x, z) = *position;
        let chunk_seed = get_chunk_seed(start_seed, x, z);
        //if either bits 25 or 26 are 1, return false
        if (chunk_seed >> 24 & 1 == 1) || (chunk_seed >> 25 & 1 == 1)
        {
            return false;
        }
    }
    true
}

fn are_all_bits_valid(positions: &[(i32, i32)], start_seed: u64) -> bool {
    for position in positions {
        let (x, z) = *position;
        let chunk_seed = get_chunk_seed(start_seed, x, z);
        if mc_first_is_zero(chunk_seed) != true {
            return false;
        }
    }
    true
}

/*
fn load_polymineo() -> Vec<(i32, i32)> {
    /*
        we can (relatively) easily find a predefined set of tiles out of all seeds.
        we are looking for the most connected tiles of mushroom, but cant easily count tiles.
        instead of counting all tiles that are connected, we will simply
    */
    let file_name = "poly_fixed_8.txt";

}
*/

fn main() {

    //load a bunch of these from a list of fixed polymineos
    let positions = [

        //bridge
        (1 + 32, 0),
        (2 + 32, 0),
        (3 + 32, 0),

        //down
        (0 + 32, 1),
        (0 + 32, 2),

        (2 + 32, 1),
        (2 + 32, 2),

        (4 + 32, 1),
        (4 + 32, 2),

    ];

    let lower_bit_count = 26;
    let upper_bit_count = 64 - lower_bit_count;

    let mut partial_canidates: Vec<u64> = Vec::new();
    for i in 0..(1 << lower_bit_count) {
        if are_lower_bits_valid(&positions, i) {
            partial_canidates.push(i);
        }
    }
    //we now have some good partial_canidates to work with
    println!("{} partial canidates found. starting full start_seed bruteforce", partial_canidates.len());

    let mut full_canidates: Vec<u64> = Vec::new();
    for (index, partial_canidate) in partial_canidates.iter().enumerate() {

        //hottest loop here
        for i in 0..(1 << upper_bit_count) {
            let current_seed = (i << lower_bit_count) | partial_canidate;
            if are_all_bits_valid(&positions, current_seed) {
                full_canidates.push(current_seed);
                //normal
                //println!("{}", current_seed as i64);
                if reverse_start_seed(current_seed).len() == 0 { continue; }
                println!("{}", reverse_start_seed(current_seed)[0] as i64);
            }
        }
        println!("partial canidate finished {index}/{} ({}%)", partial_canidates.len(), (index as f64 / partial_canidates.len() as f64) * 100.0);
    }

    println!("finished bruteforce. reversal time...");

    let world_seeds = reverse_start_seed_vec(full_canidates);

    for world_seed in world_seeds {
        println!("{}", world_seed as i64);
    }
}