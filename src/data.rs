use std::{fs::{self}, path::PathBuf};

#[derive(Clone, Copy,PartialEq, PartialOrd,Debug,Default)]
pub struct InputLine {
	pub grid_idx: i32,
	pub area1: i32,
	pub area2: i32,
	pub perc_area2: f32,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum SampleOrder {
	AB51,
	BA15,
	Unknown,
}//end enum SampleOrder

impl SampleOrder {
	pub fn from_file_id(file_id: &str) -> SampleOrder {
		let file_components: Vec<&str> = file_id.split('-').collect();
		if file_components.contains(&"up") || file_components.contains(&"uc") {SampleOrder::AB51}
		else if file_components.contains(&"dn") || file_components.contains(&"dc") {SampleOrder::BA15}
		else {SampleOrder::Unknown}
	}//end from_file_id
}//end impl for SampleOrder

#[derive(Clone,PartialEq,PartialOrd,Debug)]
pub struct InputFile {
	pub file_id: String,
	pub input_lines: Vec<InputLine>,
	pub sample_ordering: SampleOrder,
}//end struct InputFile

impl InputFile {
	pub fn new(file_id: &str, input_lines: Vec<InputLine>) -> InputFile {
		InputFile {file_id: file_id.to_string(), input_lines, sample_ordering: SampleOrder::AB51}
	}//end new()
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
		let cols: Vec<&str> = line.split(',').collect();
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
		if cols.len() < 5 {println!("{:?}",cols); continue;}
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
