extern crate rand;
use rand::thread_rng;
use rand::distributions::{Normal, IndependentSample, Range};
use std::collections::{HashMap};

extern crate linked_list as linkedlist;
use linkedlist::LinkedList;

#[derive(Debug)]
struct Population {
	cells: Vec<Deme>,
	
	col: u16,
	row: u16
}

const CARRY_SIZE:usize = 5;
const MIN_RU:i8 = 0;
const MAX_RU:i8 = 20;

impl Population {
	pub fn new(row:u16, col:u16) -> Population {
		let mut cells: Vec<Deme> = vec![];
		
		let total:u32 = col as u32 * row as u32;
		
		for _ in 0..total {
			cells.push (Deme::new())
		}
		
		Population {
			cells: cells,
			col: col,
			row: row
		}
	}
	
	fn get(&mut self, row:u16, col:u16) -> &mut Deme {
		//println!("getting {}", (row + col * self.row) as usize);
		self.cells.iter_mut().nth((row + col * self.row) as usize).unwrap()
	}
	
	fn migrate_offspring(&mut self) {
		let pct_range = Range::new(0, 400);
		
		let mut mig_map2:HashMap<(u16,u16),LinkedList<DemeEntry>> = HashMap::new();
		
		let col_max = self.col;
		let row_max = self.row;
		
		for col in 0..col_max {
			for row in 0..row_max {
				//println!("migrating ({}, {})", row, col);
				let mut last:&mut Deme = self.get(row, col);
				
				let mut cur = last.offspring.cursor();
				
				while !cur.peek_next().is_none() {
					match pct_range.ind_sample(&mut rand::thread_rng()) {
						1 => if col > 0 { mig_map2.entry((row, col-1)).or_insert(LinkedList::new()).push_front(cur.remove().unwrap()) },
						2 => if row > 0 { mig_map2.entry((row-1,col)).or_insert(LinkedList::new()).push_front(cur.remove().unwrap()) },
						3 => if col < col_max-1 { mig_map2.entry((row,col+1)).or_insert(LinkedList::new()).push_front(cur.remove().unwrap()) },
						4 => if row < row_max-1 { mig_map2.entry((row+1,col)).or_insert(LinkedList::new()).push_front(cur.remove().unwrap()) },
						_ => cur.seek_forward(1),
					}
				}
			}
		}
				
		for ((row, col), mut deme_list) in mig_map2.into_iter() {
			self.get(row, col).offspring.append(&mut deme_list);
		}
	}
	
	pub fn next_gen(&mut self) {
		for deme in self.cells.iter_mut() {
			deme.procreate();
		}
		
		self.migrate_offspring();
		
		for deme in self.cells.iter_mut() {
			deme.cut_carry();
		}
	}
}

#[derive(Debug)]
struct Deme {
	parents: LinkedList<DemeEntry>,
	offspring: LinkedList<DemeEntry>,
}

#[derive(Debug)]
struct DemeEntry {
	id: u16,
	ru: i8
}

//Copied from http://www.piston.rs/image/src/image/math/utils.rs.html#13-18 
#[inline]
pub fn clamp<N>(a: N, min: N, max: N) -> N
where N: PartialOrd {
    if a < min { return min }
    if a > max { return max }
    a
}

impl Deme {
	pub fn new() -> Deme {
		Deme {
			parents: Deme::make_gen(),
			offspring: LinkedList::new()
		}
	}

	pub fn make_gen() -> LinkedList<DemeEntry> {
		let mut gen: LinkedList<DemeEntry> = LinkedList::new();
		
		// mean 2, standard deviation 3
		let normal = Normal::new((MAX_RU-MIN_RU) as f64/2 as f64, (MAX_RU-MIN_RU) as f64/7 as f64);
		
		for _ in 1..CARRY_SIZE {
			let ru = clamp(normal.ind_sample(&mut rand::thread_rng()), MIN_RU as f64, MAX_RU as f64);
			gen.push_back(DemeEntry{id: 0, ru: ru as i8})
		}
		
		gen
	}
	
	pub fn procreate(&mut self) {
		let mut new_gen: LinkedList<DemeEntry> = LinkedList::new();
		
		let pct_range = Range::new(0, 100);
		
		for entry in self.parents.iter_mut() {
			
			for _ in 0..2 {
				let pct:i8 = pct_range.ind_sample(&mut rand::thread_rng());
				let adjust = match pct {
					1 => 1,
					2 => -1,
					_ => 0
				};
				//println!("ru: {}, adjust: {}", entry.ru, adjust);
				let new_ru = clamp(entry.ru + adjust, MIN_RU, MAX_RU);
				
				new_gen.push_back(DemeEntry { id:0, ru: new_ru })
			}
		}
		
		self.offspring = new_gen
	}
	
	fn cut_carry(&mut self) {
		self.offspring.split_at(CARRY_SIZE);
		self.parents = self.offspring.split_at(0);
	}
}

fn main() {
	let mut pop = Population::new(2, 1);
    
    pop.next_gen();
    pop.next_gen();
    pop.next_gen();
    
    
    println!("pop: {:?}", pop);
}
