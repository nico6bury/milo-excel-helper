use std::{path::PathBuf, slice::Iter};

use rust_xlsxwriter::{Format, FormatAlign, Workbook, XlsxError};

use crate::data::{InputFile, InputLine, SampleOrder};

#[derive(Clone, Debug, PartialEq)]
pub enum DataVal {
	String(String),
	Integer(i32),
	Float(f32),
}

impl DataVal {
	pub fn str(str: &str) -> DataVal {DataVal::String(str.to_string())}
}

/// for each in header:
/// - name of header
/// - decimal places to display for header
/// - true if this header contains percent values
/// for each in sample_row:
/// - A row of data. for each in row of data:
/// 	- individual cells of data
#[derive(Clone, Debug, PartialEq)]
pub struct DataChunk{ 
	pub headers: Vec<(String, usize, bool)>,
	pub rows: Vec<Vec<DataVal>>
}

impl DataChunk {
	pub fn new() -> DataChunk {
		DataChunk {
			headers: Vec::new(),
			rows: Vec::new(),
		}
	}
}

pub fn get_workbook() -> Workbook {
	Workbook::new()
}

pub fn close_workbook(workbook: &mut Workbook, output_path: &PathBuf) -> Result<(),XlsxError> {
	workbook.save(output_path)?;
	Ok(())
}

pub fn extract_labelled_chunks(data: &Vec<InputFile>) -> Vec<DataChunk> {
	let mut chunks = Vec::new();
	for file in data {
		let mut chunk = DataChunk::new();
		chunk.headers.push(("Sample".to_string(),0, false));
		chunk.headers.push(("FileID".to_string(),0, false));
		chunk.headers.push(("GridIdx".to_string(),0, false));
		chunk.headers.push(("Area1".to_string(),0, false));
		chunk.headers.push(("Area2".to_string(),0, false));
		chunk.headers.push(("%Area2".to_string(),1, false));

		let sample_labels = file.sample_ordering.get_labels();
		for (i,line) in file.input_lines.iter().enumerate() {
			let label = sample_labels.get(i).unwrap_or(&"???");
			chunk.rows.push(vec![
				DataVal::str(label),
				DataVal::str(&file.file_id),
				DataVal::Integer(line.grid_idx),
				DataVal::Integer(line.area1),
				DataVal::Integer(line.area2),
				DataVal::Float(line.perc_area2)
			]);
		}//end going over each line

		chunks.push(chunk);
	}//end looping over files in data
	return chunks;
}//end extract_labelled_chunks()

pub fn extract_sorted_chunks_1(data: &Vec<InputFile>) -> Vec<DataChunk> {
	// each chunk is an ordering, so ab51 or ba15
	// thus, before creating chunks, must sort out input by ordering
	// sorting input by ordering can be done in one line with .iter.filter.collect, so
	// we should create an inner function which just returns a chunk for a list of
	// input files, and then we can call that super easily on any sorting we need

	fn extract_sorted_chunk_1_helper(files: &Vec<&InputFile>) -> DataChunk {
		let mut chunk = DataChunk::new();

		// add the headers to chunk
		chunk.headers.push(("Sample".to_string(),0, false));
		// // add Area1,Area2,%Area2 for each file, then av, std, cv
		for _ in 0..(files.len()/* + 3*/) {
			chunk.headers.push(("".to_string(),0, false));
			chunk.headers.push(("Area1".to_string(),0, false));
			chunk.headers.push(("Area2".to_string(),0, false));
			chunk.headers.push(("%Area2".to_string(),1, false));
		}//end adding Area headers

		// print out data in columns instead of rows
		let sample_labels = SampleOrder::AB15.get_labels();
		for sample in sample_labels 
		{ chunk.rows.push(vec![DataVal::str(sample)]); }
		
		// stuff for area, std, cv
		let mut last_line = vec![DataVal::str("FileID")];
		let empty = DataVal::str("");
		
		for file in files {
			for (row_idx, row) in InputFile::get_ab15_order(file.sample_ordering, &file.input_lines).iter().enumerate() {
				let a1 = DataVal::Integer(row.area1);
				let a2 = DataVal::Integer(row.area2);
				let a2p = DataVal::Float(row.perc_area2);
				match chunk.rows.get_mut(row_idx) {
					Some(row) => row.append(&mut vec![empty.clone(),a1,a2,a2p]),
					None => chunk.rows.push(vec![DataVal::str("?"),empty.clone(),a1,a2,a2p]),
				}//end adding data to chunk, regardless of whether we have sample
			}//end looping over the lines of data in this file
			last_line.push(empty.clone());
			last_line.push(empty.clone());
			last_line.push(DataVal::str(&file.file_id));
			last_line.push(empty.clone());
		}//end adding data to chunk for each file
		chunk.rows.push(last_line);

		// TODO: Add average, stdev, csv

		return chunk;
	}//end extract_chunk_1_helper()

	// get vector of unique SampleOrder values
	let mut all_orderings = data.iter()
		.map(|f| f.sample_ordering)
		.collect::<Vec<SampleOrder>>();
	all_orderings.sort_unstable();
	all_orderings.dedup();

	let mut chunks = Vec::new();

	for ordering in all_orderings {
		let files = data.iter()
			.filter(|f| f.sample_ordering == ordering)
			.collect();
		let chunk = extract_sorted_chunk_1_helper(&files);
		chunks.push(chunk);
	}//end getting a chunk of files for each ordering we found

	return chunks;
}//end extract_sorted_chunks_1()

pub fn extract_sorted_chunks_2(data: &Vec<InputFile>) -> Vec<DataChunk> {
	// simple little functions to avoid repeating things
	fn i_to_val(input: &InputLine, idx: i32) -> DataVal {
		match idx {
			1 => DataVal::Integer(input.area1),
			2 => DataVal::Integer(input.area2),
			0 => DataVal::Float(input.perc_area2),
			_ => DataVal::str("????")
		}//end matching index to property we want
	}//end i_to_val
	fn i_to_label(idx: i32) -> String {
		match idx {
			1 => "Area1".to_string(),
			2 => "Area2".to_string(),
			0 => "%Area2".to_string(),
			_ => "Unknown".to_string()
		}//end matching index to column label
	}//end i_to_label
	
	let mut chunks = Vec::new();

	for i in 0..=2 {
		let mut chunk = DataChunk::new();
		// add the headers
		chunk.headers.push(("Sample".to_string(),0, false));
		for _ in data.iter()
		{ chunk.headers.push((i_to_label(i),1, false)); }

		// add the data
		let sample_labels = SampleOrder::AB15.get_labels();
		sample_labels
			.iter()
			.map(|elem| DataVal::str(elem))
			.for_each(|elem| chunk.rows.push(vec![elem]));
		let mut last_line = vec![DataVal::str("FileID")];
		for (col_idx, file) in data.iter().enumerate() {
			for (row_idx, row) in InputFile::get_ab15_order(file.sample_ordering, &file.input_lines).iter().enumerate() {
				let this_row = match chunk.rows.get_mut(row_idx) {
					Some(chunk_row) => chunk_row,
					None => {
						while !(row_idx < chunk.rows.len()) {
							let mut new_placeholder_row = Vec::new();
							for _ in 0..(col_idx+1)
							{ new_placeholder_row.push(DataVal::str("??")) }
							chunk.rows.push(new_placeholder_row);
						}//end populating empty space so we're in the right position
						chunk.rows.last_mut().unwrap()
					}//end case that we need to create a row to reference
				};// end getting reference for this row
				this_row.push(i_to_val(row, i));
			}//end looping over rows in the file
			last_line.push(DataVal::str(&file.file_id));
		}//end looping over files
		chunk.rows.push(last_line);

		chunks.push(chunk);
	}//end looping over val indices

	return chunks;
}//end extract_sorted_chunks_2()

pub fn extract_sum_chunk(data: &Vec<InputFile>) -> DataChunk {
	let mut chunk = DataChunk::new();
	// add the headers
	chunk.headers.push(("Sample".to_string(),0, false));
	for _ in data.iter()
	{ chunk.headers.push(("%Area2".to_string(),1, false)); }
	chunk.headers.push(("".to_string(),0, false));
	chunk.headers.push(("Avg".to_string(),1, false));
	chunk.headers.push(("Std".to_string(),2, false));
	chunk.headers.push(("CV".to_string(),2, true));
	
	// add sample labels
	let sample_labels = SampleOrder::AB15.get_labels();
	sample_labels.iter()
		.map(|lbl| DataVal::str(lbl))
		.for_each(|lbl| chunk.rows.push(vec![lbl]));

	// add %Area2 for each file
	for (col_idx, file) in data.iter().enumerate() {
		for (line_idx, line) in 
		InputFile::get_ab15_order(
			file.sample_ordering,
			&file.input_lines
		).iter().enumerate() {
			// gets reference to current row. Error validation in case of unlabelled samples
			let this_chunk_row_ref = match chunk.rows.get_mut(line_idx) {
				Some(chunk_row) => chunk_row,
				None => {
					while !(line_idx < chunk.rows.len()) {
						let mut new_placeholder_row = Vec::new();
						for _ in 0..(col_idx+1)
						{ new_placeholder_row.push(DataVal::str("??")); }
						chunk.rows.push(new_placeholder_row);
					}//end populating empty space so we're in the right position
					chunk.rows.last_mut().unwrap()
				}//end case that we need to create a row to reference
			};//end getting reference for this row
			this_chunk_row_ref.push(DataVal::Float(line.perc_area2));
		}//end looping over lines in the file
	}//end looping over files to include

	// Add avg, std, cv
	for row in chunk.rows.iter_mut() {
		let data_slice: &Vec<f32> = &row[1..].iter()
			.filter_map(|d| match d {
				DataVal::Float(f) => Some(f),
				_ => None,})
			.map(|f| f.clone())
			.collect::<Vec<f32>>();
		row.push(DataVal::str(""));
		row.push(DataVal::Float(crate::math::avg(data_slice)));
		row.push(DataVal::Float(crate::math::std(data_slice)));
		row.push(DataVal::Float(crate::math::cv(data_slice)));
	}//end adding avg, std, cv for each row

	return chunk;
}//end extract_sorted_chunks_3()

/// Writes a number of chunks of data to a sheet in a workbook
pub fn write_chunks_to_sheet(
	workbook: &mut Workbook,
	chunks: Iter<DataChunk>,
	sheet_name: &str
) -> Result<(),XlsxError> {
	// set up a new sheet in the workbook
	let sheet = workbook.add_worksheet();
	sheet.set_name(sheet_name)?;
	
	// create a few formats to use later
	let bold = Format::new().set_bold().set_align(FormatAlign::Center);
	let default_format = Format::new().set_align(FormatAlign::Center);

	// actually start writing all the data to everything
	let mut chunk_row = 0;
	for chunk in chunks {
		// write the header row
		for (index, header) in chunk.headers.iter().enumerate() {
			let index = index as u16;
			sheet.write_with_format(
				chunk_row,
				index,
				header.0.clone(),
				&bold
			)?;
		}//end writing each header to one row

		// create formats for each header row, based on chunk info
		let mut formats = Vec::new();
		for (_,decimals,is_percent) in chunk.headers.iter() {
			let mut num_format = String::from("0.");
			for _ in 0..*decimals {num_format.push_str("0");}
			if *is_percent {num_format.push_str("%");}
			let this_format = Format::new()
				.set_num_format(num_format)
				.set_align(FormatAlign::Center);
			formats.push(this_format);
		}//end creating format for each header

		// actually get around to writing the data for this chunk
		chunk_row += 1;
		for row in chunk.rows.iter() {
			for (col_offset, value) in row.iter().enumerate() {
				let format = formats.get(col_offset).unwrap_or(&default_format);
				let col_offset = col_offset as u16;
				match value {
					DataVal::Integer(i) => sheet.write_number_with_format(chunk_row,col_offset,*i as f64,&default_format)?,
					DataVal::Float(f) => sheet.write_number_with_format(chunk_row,col_offset, *f, format)?,
					DataVal::String(s) => sheet.write_with_format(chunk_row, col_offset, s,&default_format)?,
				};//end matching type of data
			}//end looping over cells within row
			chunk_row += 1;
		}//end looping over the rows for this chunk

		// loop maintenance for writing multiple chunks
		chunk_row += 2;
	}//end writing each chunk of data to the sheet

	Ok(())
}
