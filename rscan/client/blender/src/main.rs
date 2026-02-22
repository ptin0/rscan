use std::{fs::File, io::Write};

use binwrite::BinWrite;
use rand::Rng;
use rand::distributions::{Distribution};


#[derive(BinWrite)]
#[binwrite(little)]
struct Data {
    step_size: u8,  // 0
    line_size: u8,  // 1
    line_count: u8, // 2
    step_count: u8, // 3
    line_start: u8, // 4
    step_start: u8, // 5
    #[binwrite(big)]
    data: Vec<Vec<u32>>,
}

impl Data {
    fn new(
        step_size: u8,
        line_size: u8,
        line_count: u8,
        step_count: u8,
        line_start: u8,
        step_start: u8
    ) -> Self {

        Data {
            step_size,
            line_size,
            line_count: line_count,
            step_count,
            line_start,
            step_start,
            data: generate_points(line_count, step_count)}
    }
}

fn generate_points(
    steps: u8,
    lines: u8) -> Vec<Vec<u32>> {
    
    let mut cnt: usize = 0;

    let mut mesh = Vec::<Vec<u32>>::new();
    
    let mut rng = rand::thread_rng();
    let normal = rand_distr::Normal::<f32>::new(1000.0, 25.0).unwrap();

    for _line_number in 0..=lines-1 {
        let mut line = Vec::<u32>::new();
        for _point in 0..steps {
            line.push(normal.sample(&mut rng) as u32);
            cnt += 1;
        }
        mesh.push(line);

        let mut line = Vec::<u32>::new();
        for _point in steps-1..=0 {
            line.push(normal.sample(&mut rng) as u32);
            cnt += 1;
        }
        mesh.push(line);
    }
    println!("Point Count {:?}", cnt);
    mesh
}

fn main() {
    let test = Data::new(1, 1, 30, 60, 0, 0);
    let mut bytez = vec![];
    test.write(&mut bytez).unwrap();

    let mut file = File::create_new("testfiler.dat").unwrap();
    file.write(&bytez).unwrap();

    println!("{:?}", bytez);
}
