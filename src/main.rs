extern crate time;
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

#[derive(Debug)]
struct Deme {
	parents: LinkedList<DemeEntry>,
	offspring: LinkedList<DemeEntry>,
}

#[derive(Debug)]
struct DemeEntry {
	id: u32,
	ru: i8
}

const CARRY_SIZE:usize = 100;
const MIN_RU:i8 = 1;
const MAX_RU:i8 = 19;

//Copied from http://www.piston.rs/image/src/image/math/utils.rs.html#13-18 
#[inline]
pub fn clamp<N>(a: N, min: N, max: N) -> N
where N: PartialOrd {
    if a < min { return min }
    if a > max { return max }
    a
}

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
		let pct_range = Range::new(0,400);
		
		//All elements in given linked list should be migrated to the cell
		//indicated by the key
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
						1 => if col > 0 {
								mig_map2.entry((row, col-1)).or_insert(LinkedList::new()).push_front(cur.remove().unwrap())
							},
						2 => if row > 0 {
								mig_map2.entry((row-1,col)).or_insert(LinkedList::new()).push_front(cur.remove().unwrap())
							},
						3 => if col < col_max-1 {
								mig_map2.entry((row,col+1)).or_insert(LinkedList::new()).push_front(cur.remove().unwrap())
							},
						4 => if row < row_max-1 {
								mig_map2.entry((row+1,col)).or_insert(LinkedList::new()).push_front(cur.remove().unwrap())
							},
						_ => cur.seek_forward(1),
					}
				}
			}
		}
				
		for ((row, col), mut deme_list) in mig_map2.into_iter() {
			let mut deme = self.get(row, col);
			let mut offspring_cur = deme.offspring.cursor();
			let mut new_cur = deme_list.cursor();
			
			//XXX: TODO make this more efficient
			while !new_cur.peek_next().is_none() {
				let deme_entry_id:u32 = new_cur.peek_next().unwrap().id;
				
				let mut insert = false;
				
				while !insert {
					
					{
						let curr = offspring_cur.next();
					
						match curr {
							None => {
									insert = true
								},
							Some(existing) =>
								if deme_entry_id < existing.id {
									insert = true;
								},
						}
					}
					
					if insert {
						offspring_cur.insert(new_cur.remove().unwrap());
					}
				}
			}
			
		}
	}
	
	pub fn next_gen(&mut self) {
		//println!("Starting next_gen");
		
		for deme in self.cells.iter_mut() {
			deme.procreate();
		}
		
		//println!("Done procreating");
		
		self.migrate_offspring();
		
		//println!("Migrating offspring");
		
		for deme in self.cells.iter_mut() {
			deme.cut_carry();
		}
		
		//println!("Cut carry");
	}
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
		
		// 
		let id_range = Range::new(0, std::u32::MAX);
		let mut ids: Vec<u32> = vec![];
		
		for _ in 1..CARRY_SIZE {
			ids.push(id_range.ind_sample(&mut rand::thread_rng()))
		}
		ids.sort();
		
		let mut idx_iter = ids.iter();
		
		// mean 2, standard deviation 3
		let normal = Normal::new((MAX_RU-MIN_RU) as f64/2 as f64, (MAX_RU-MIN_RU) as f64/7 as f64);
		
		for _ in 1..CARRY_SIZE {
			let id = idx_iter.next().unwrap().clone();
			let ru = clamp(normal.ind_sample(&mut rand::thread_rng()), MIN_RU as f64, MAX_RU as f64);
			gen.push_back(DemeEntry{id: id, ru: ru as i8})
		}
		
		gen
	}
	
	pub fn procreate(&mut self) {
		let mut new_gen: Vec<DemeEntry> = vec![];
		
		let pct_range = Range::new(0, 200);
		
		let id_range = Range::new(0, std::u32::MAX);
		
		for entry in self.parents.iter_mut() {
			
			for _ in 0..2 {
				let pct:i16 = pct_range.ind_sample(&mut rand::thread_rng());
				let adjust = match pct {
					1 => 1,
					2 => -1,
					_ => 0
				};
				//println!("ru: {}, adjust: {}", entry.ru, adjust);
				let new_ru = clamp(entry.ru + adjust, MIN_RU, MAX_RU);
				
				let new_id = id_range.ind_sample(&mut rand::thread_rng());
				
				new_gen.push(DemeEntry { id:new_id, ru: new_ru })
			}
		}
		
		new_gen.sort_by(|a, b| {a.id.cmp(&b.id)});
		
		let mut new_gen_list = LinkedList::new();
		
		for entry in new_gen.into_iter() {
			new_gen_list.push_back(entry)
		}
		
		self.offspring = new_gen_list
	}
	
	fn cut_carry(&mut self) {
		self.offspring.split_at(CARRY_SIZE);
		self.parents = self.offspring.split_at(0);
	}
}

fn main() {
	
	let start_sec = time::get_time().sec;
	
	let mut pop = Population::new(25, 15);
    
    for _ in 1..100 {
	    pop.next_gen();
	    //println!("pop: {:#?}", pop);
    }
    
    println!("Completed in {} seconds", (time::get_time().sec - start_sec));
}
