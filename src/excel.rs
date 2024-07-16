use std::{ops::Sub, path::PathBuf, slice::Iter};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputVal {
	KernelArea,
	EndospermArea,
	PercentArea,
}//end enum OutputVal

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

pub fn extract_sum_chunk(data: &Vec<InputFile>, output_val: OutputVal) -> DataChunk {
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
			match output_val {
				OutputVal::KernelArea => this_chunk_row_ref.push(DataVal::Float(line.area1 as f32)),
				OutputVal::EndospermArea => this_chunk_row_ref.push(DataVal::Float(line.area2 as f32)),
				OutputVal::PercentArea => this_chunk_row_ref.push(DataVal::Float(line.perc_area2)),
			}//end matching the kind of sum to do
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
}//end extract_sum_chunk()

pub fn extract_stats_chunk(data: &Vec<InputFile>, output_val: OutputVal) -> DataChunk {
	let mut chunk = DataChunk::new();
	// add the headers
	chunk.headers.push(("Sample".to_string(),1,false));
	chunk.headers.push(("Avg".to_string(),1,false));
	chunk.headers.push(("Std".to_string(),1,false));
	chunk.headers.push(("CV".to_string(),1,true));
	chunk.headers.push(("".to_string(),1,false));
	chunk.headers.push(("Split Diff".to_string(),1,false));
	chunk.headers.push(("Split Std".to_string(),1,false));
	chunk.headers.push(("Split Avg".to_string(),1,false));
	chunk.headers.push(("Split CV".to_string(),1,true));

	// add sample labels, also having overall sample, like ag05-1a
	let sample_labels = SampleOrder::AB15.get_labels();
	let filenames: Vec<&str> = data.iter()
		.map(|file| file.file_id.as_str())
		.collect();
	let mut common_sample_id = guess_sample_id(&filenames).unwrap_or("".to_string());
	if !common_sample_id.eq("") {common_sample_id += "-";}
	sample_labels.iter()
		.map(|lbl| DataVal::String(common_sample_id.clone() + lbl))
		.for_each(|lbl| chunk.rows.push(vec![lbl]));

	// collect %Area2 for each file
	// rows_per_sample has each column from file, each row from sample
	// each inner vec is one row, iterate through one row for cols
	let mut rows_per_sample: Vec<Vec<f32>> = Vec::new();
	for (col_idx, file) in data.iter().enumerate() {
		for (line_idx, line) in
		InputFile::get_ab15_order(
			file.sample_ordering,
			&file.input_lines
		).iter().enumerate() {
			let this_row_ref = match rows_per_sample.get_mut(line_idx) {
				Some(row_ref) => row_ref,
				None => {
					while !(line_idx < rows_per_sample.len()) {
						let mut new_placeholder_row = Vec::new();
						for _ in 0..(col_idx)
						{ new_placeholder_row.push(-1.); }
						rows_per_sample.push(new_placeholder_row);
					}//end populating empty space so we're in the right position
					rows_per_sample.last_mut().expect("We just added to the vec, it shouldn't be empty!")
				}//end case that we need to create a row to reference
			};//end getting reference for this row
			match output_val {
				OutputVal::KernelArea => this_row_ref.push(line.area1 as f32),
				OutputVal::EndospermArea => this_row_ref.push(line.area2 as f32),
				OutputVal::PercentArea => this_row_ref.push(line.perc_area2),
			}
		}//end looping over samples in ab15 order
	}//end looping over files

	// Add avg, std, cv per file
	for i in 0..(rows_per_sample.len()) {
		// get references for right space in vecs
		let this_data_row_ref = rows_per_sample.get(i);
		if this_data_row_ref.is_none() {continue;}
		let this_data_row_ref = this_data_row_ref.unwrap();
		let this_chunk_row_ref = match chunk.rows.get_mut(i) {
			Some(chunk_row) => chunk_row,
			None => {
				while !(i < chunk.rows.len()) {
					let mut new_placeholder_row = Vec::new();
					for _ in 0..(i+1)
					{ new_placeholder_row.push(DataVal::str("??")); }
					chunk.rows.push(new_placeholder_row);
				}//end populating empty space so we're in the right position
				chunk.rows.last_mut().expect("We just added to the vec; it shouldn't be empty!")
			}//end case that we need to create a row to reference
		};//end getting reference for this row in chunk

		// add per-sample (1a, 1b, 2a, 2b, etc) data
		let avg = crate::math::avg(this_data_row_ref);
		let std = crate::math::std(this_data_row_ref);
		let cv = crate::math::cv(this_data_row_ref);
		this_chunk_row_ref.push(DataVal::Float(avg));
		this_chunk_row_ref.push(DataVal::Float(std));
		this_chunk_row_ref.push(DataVal::Float(cv));
		this_chunk_row_ref.push(DataVal::str(""));

		// add sample average data (per 1, 2, etc)
		if i % 2 == 0 {
			let sa = rows_per_sample.get(i);
			let sb = rows_per_sample.get(i+1);
			if sa.is_some() && sb.is_none() {this_chunk_row_ref.append(&mut vec![DataVal::str(""),DataVal::str(""),DataVal::str(""),DataVal::str("")])}
			else if sa.is_some() && sb.is_some() {
				let sa = sa.expect("We already checked sa is_some()!?");
				let sb = sb.expect("We already checked sb is_some()!?");
				let sa_avg = crate::math::avg(sa);
				let sb_avg = crate::math::avg(sb);
				let s_diff = sa_avg.sub(sb_avg).abs();
				let s_std = crate::math::std(&vec![sa_avg,sb_avg]);
				let s_avg = (sa_avg + sb_avg) / 2.;
				let s_cv = s_std / s_avg;
				// split diff, split std, split avg, split cv
				this_chunk_row_ref.push(DataVal::Float(s_diff));
				this_chunk_row_ref.push(DataVal::Float(s_std));
				this_chunk_row_ref.push(DataVal::Float(s_avg));
				this_chunk_row_ref.push(DataVal::Float(s_cv));
			}//end if we found both a and b
		}//end if we're on an even index
	}//end looping matching file data to chunk rows for each file

	return chunk;
}//end extract_stats_chunk()

/// Assuming a set of filenames has the same sample id,
/// and assuming that that id is separated by dashes,
/// attempts to find a common sample id from a list of
/// names.  
/// If multiple matches are found, will prefer the
/// match that includes numbers over one without.  
/// If no matches are found, returns none.
/// If multiple matches are found, returns all of the matches,
/// separated by dashes.  
/// If the given vec is empty, returns none.
/// 
/// # Examples
/// ```
/// use milo_excel_helper::excel::guess_sample_id;
/// let filenames = vec!["ns-ag05-ab15.tif","ns-ag05-ba51","ns-ag05-ab15"];
/// let guessed_id = guess_sample_id(&filenames);
/// let expected_id = "ag05".to_string();
/// assert_eq!(guessed_id.unwrap(), expected_id);
/// ```
/// ```
/// use milo_excel_helper::excel::guess_sample_id;
/// let filenames = vec!["ns-ag05-131-ab15.tif","ns-ag05-132-ab15.tif"];
/// let guessed_id = guess_sample_id(&filenames);
/// let expected_id = "ns-ag05-ab15-tif".to_string();
/// assert_eq!(guessed_id.unwrap(), expected_id);
/// ```
/// ```
/// use milo_excel_helper::excel::guess_sample_id;
/// let filenames = vec![
/// 	"ns-ag05-131-ab15.tif",
/// 	"ns-ag05-132-ba51.tif",
/// 	"ns-ag05-133-ab15.tif",
/// 	"ns-ag05-134-ba51.tif",
/// 	"ns-ag05-135-ab15.tif",
/// 	"ns-ag05-136-ba51.tif"
/// ];
/// let guessed_id = guess_sample_id(&filenames);
/// let expected_id = "ag05".to_string();
/// assert_eq!(guessed_id.unwrap(), expected_id);
/// ```
/// ```
/// use milo_excel_helper::excel::guess_sample_id;
/// let filenames = vec![
/// 	"ns-ag05-131-ab15.tif",
/// 	"ns-ag05-132-ba51.tif",
/// 	"ns-ag05-133-ab15.tif",
/// 	"ns-ag05-134-ba51.tif",
/// 	"ns-ag05-135-ab15.tif",
/// 	"ns-ag05-136-ba51.tif",
/// 	"ns-ag06-141-ab15.tif"
/// ];
/// let guessed_id = guess_sample_id(&filenames);
/// let expected_id = "ns-tif".to_string();
/// assert_eq!(guessed_id.unwrap(), expected_id);
/// ```
pub fn guess_sample_id(filenames: &Vec<&str>) -> Option<String> {
	if filenames.len() < 1 {return None;}
	let first_id = filenames.first().expect("There should be a first filename; We already checked that len() is 1 or more.");
	let first_components = first_id.split(&['-','.']);
	// get list of components contained within every filename
	let full_matches: Vec<&str> = first_components
		.filter(
			|id| filenames.iter().find(|name| !name.contains(id)).is_none()
		)
		.collect();
	match full_matches.len() {
		0 => return None,
		1 => return Some(full_matches.first().unwrap().to_string()),
		_ => {
			let matches_with_nums: Vec<&&str> = full_matches.iter()
				.filter(|id| id.chars().find(|char| char.is_numeric()).is_some())
				.collect();
			match matches_with_nums.len() {
				1 => return Some(matches_with_nums.first().unwrap().to_string()),
				_ => return Some(full_matches.join("-"))
			}
		}//end case of more than 1 match
	}//end matching based on number of matches
}//end guess_sample_id()

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
