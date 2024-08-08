use std::{fs::{self}, path::PathBuf};

#[derive(Clone, Copy,PartialEq, PartialOrd,Debug,Default)]
pub struct InputLine {
	pub grid_idx: i32,
	pub area1: i32,
	pub area2: i32,
	pub perc_area2: f32,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Eq, Ord)]
#[non_exhaustive]
pub enum SampleOrder {
	AB51,
	BA15,
	AB15,
	BA51,
	/// AB 1-10
	AB110,
	/// BA 10-1
	BA101,
	Unknown,
}//end enum SampleOrder

impl SampleOrder {
	pub fn from_file_id(file_id: &str) -> SampleOrder {
		let file_components: Vec<&str> = file_id.split(&['-','.']).collect();

		let ab51_indic = vec!["up","uc","51ab","ab51","51AB","AB51"];
		let ba15_indic = vec!["dn","dc","15ba","ba15","15BA","BA15"];
		let ab15_indic = vec!["UP","ab15","15ab","AB15","15AB","top"];
		let ba51_indic = vec!["DN","ba51","51ba","BA51","51BA","btm"];
		let ab110_indic = vec!["ab110","AB110","ab1_10","AB1_10"];
		let ba101_indic = vec!["ba101","BA101","ba10_1","BA10_1"];

		let found_ab51 = file_components.iter().find(|c| ab51_indic.contains(c)).is_some();
		let found_ba15 = file_components.iter().find(|c| ba15_indic.contains(c)).is_some();
		let found_ab15 = file_components.iter().find(|c| ab15_indic.contains(c)).is_some();
		let found_ba51 = file_components.iter().find(|c| ba51_indic.contains(c)).is_some();
		let found_ab110 = file_components.iter().find(|c| ab110_indic.contains(c)).is_some();
		let found_ba101 = file_components.iter().find(|c| ba101_indic.contains(c)).is_some();

		if found_ab15 {SampleOrder::AB15}
		else if found_ba51 {SampleOrder::BA51}
		else if found_ab110 {SampleOrder::AB110}
		else if found_ba101 {SampleOrder::BA101}
		else if found_ab51 {SampleOrder::AB51}
		else if found_ba15 {SampleOrder::BA15}
		else {SampleOrder::Unknown}
	}//end from_file_id

	pub fn get_labels(&self) -> Vec<&str> {
		match self {
			SampleOrder::AB51 => vec!["5a","5b","4a","4b","3a","3b","2a","2b","1a","1b"],
			SampleOrder::BA15 => vec!["1b","1a","2b","2a","3b","3a","4b","4a","5b","5a"],
			SampleOrder::AB15 => vec!["1a","1b","2a","2b","3a","3b","4a","4b","5a","5b"],
			SampleOrder::BA51 => vec!["5b","5a","4b","4a","3b","3a","2b","2a","1b","1a"],
			SampleOrder::AB110 => vec!["1a","1b","2a","2b","3a","3b","4a","4b","5a","5b","6a","6b","7a","7b","8a","8b","9a","9b","10a","10b"],
			SampleOrder::BA101 => vec!["10b","10a","9b","9a","8b","8a","7b","7a","6b","6a","5b","5a","4b","4a","3b","3a","2b","2a","1b","1a"],
			SampleOrder::Unknown => vec!["??","??","??","??","??","??","??","??","??","??"],
		}//end matching self
	}//end get_labels
}//end impl for SampleOrder

#[derive(Clone,PartialEq,PartialOrd,Debug)]
pub struct InputFile {
	pub file_id: String,
	pub input_lines: Vec<InputLine>,
	pub sample_ordering: SampleOrder,
}//end struct InputFile

impl InputFile {
	pub fn new(file_id: &str, input_lines: Vec<InputLine>) -> InputFile {
		InputFile {file_id: file_id.to_string(), input_lines, sample_ordering: SampleOrder::from_file_id(file_id)}
	}//end new()

	pub fn get_ab15_order(cur_order: SampleOrder, lines: &Vec<InputLine>) -> Vec<&InputLine> {
		match cur_order {
			SampleOrder::AB15 => return lines.iter().collect(),
			SampleOrder::BA51 => return lines.iter().rev().collect(),
			SampleOrder::AB51 | SampleOrder::BA15 => {
				let mut ab_swp: Vec<&InputLine> = Vec::new();
				let mut i = 0; let mut j = 1;
				while i < lines.len() && j < lines.len() {
					ab_swp.push(&lines[j]);
					ab_swp.push(&lines[i]);
				i+= 2; j += 2;}
				// do iterator stuff to return stuff in the right format
				let ab_swp_iter = ab_swp.iter().map(|n| *n);
				if cur_order == SampleOrder::BA15 {return ab_swp_iter.rev().collect();}
				else {return ab_swp_iter.collect()}
			},
			SampleOrder::AB110 => return lines.iter().collect(),
			SampleOrder::BA101 => return lines.iter().rev().collect(),
			SampleOrder::Unknown => return lines.iter().collect(),
		}
	}
}//end impl for InputFile

pub fn read_csv_file(file: &PathBuf) -> Option<Vec<InputFile>> {
	let mut input_files: Vec<InputFile> = Vec::new();
	let mut last_file_id;
	let mut tmp_row_data = Vec::new();

	let mut _headers: Vec<&str> = Vec::new();
	let mut header_idx = 0;

	let contents = fs::read_to_string(file).unwrap();
	let lines: Vec<&str> = contents.split('\n').collect();
	
	// get the headers and header_idx
	loop {
		let line = lines[header_idx];
		let cols: Vec<&str> = line.split(',').filter(|col| !col.eq(&"")).collect();
		if cols.len() > 5 {
			// save headers
			_headers = cols;
			// figure out the first file_id
			let n_line = lines[header_idx + 1];
			let n_cols: Vec<&str> = n_line.split(',').collect();
			last_file_id = n_cols[0].to_string();
			// exit loop, work here is done
			break;
		} else {header_idx += 1;}
	}//end looping to find headers

	// loop over lines after headers, get the data
	for i in (header_idx+1)..lines.len() {
		// just get the actual columns
		let line = lines[i];
		let cols: Vec<&str> = line.split(',').collect();
		if cols.len() < 5 {if !cols.eq(&(vec![""])) {println!("{:?}",cols)}; continue;}
		// get all the actual data
		let file_id = cols[0];
		let grid_idx: i32 = cols[1].parse().unwrap_or(-2);
		let area1: i32 = cols[2].parse().unwrap_or(-2.) as i32;
		let area2: i32 = cols[3].parse().unwrap_or(-2.) as i32;
		let perc_area2: f32 = cols[4].parse().unwrap_or(-2.);
		let new_input_line = InputLine {grid_idx,area1,area2,perc_area2,};
		// make sure we're separating files from lines
		if !file_id.eq(&last_file_id) {
			let new_input_file = InputFile::new(&last_file_id, tmp_row_data);
			input_files.push(new_input_file);
			tmp_row_data = vec![new_input_line];
			last_file_id = file_id.to_string();
		} else {
			tmp_row_data.push(new_input_line);
		}//end else we add a new line as usual
	}//end looping over indices for lines
	
	// clean up anything left and add to returned Vec
	if tmp_row_data.len() > 0 {
		let new_input_file = InputFile::new(&last_file_id,tmp_row_data);
		input_files.push(new_input_file);
	}//end if we should add the last few lines to input_files

	return Some(input_files);
}//end read_csv_file(reader)
