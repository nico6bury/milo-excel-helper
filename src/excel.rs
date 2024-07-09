use std::{path::PathBuf, slice::Iter};

use rust_xlsxwriter::{Format, Workbook, XlsxError};

#[derive(Clone, Debug, PartialEq)]
pub enum DataVal {
	String(String),
	Integer(i32),
	Float(f32),
}

/// for each in header:
/// - name of header
/// - decimal places to display for header
/// for each in sample_row:
/// - A row of data. for each in row of data:
/// 	- individual cells of data
#[derive(Clone, Debug, PartialEq)]
pub struct DataChunk{ 
	pub headers: Vec<(String, usize)>,
	pub rows: Vec<Vec<DataVal>>
}

pub fn get_workbook() -> Workbook {
	Workbook::new()
}

pub fn close_workbook(workbook: &mut Workbook, output_path: &PathBuf) -> Result<(),XlsxError> {
	workbook.save(output_path)?;
	Ok(())
}

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
	let bold = Format::new().set_bold();
	let default_format = Format::new().set_num_format("0.00");

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
		for (_,decimals) in chunk.headers.iter() {
			let mut num_format = String::from("0.");
			for _ in 0..*decimals {num_format.push_str("0");}
			let this_format = Format::new().set_num_format(num_format);
			formats.push(this_format);
		}//end creating format for each header

		// actually get around to writing the data for this chunk
		chunk_row += 1;
		for row in chunk.rows.iter() {
			for (col_offset, value) in row.iter().enumerate() {
				let format = formats.get(col_offset).unwrap_or(&default_format);
				let col_offset = col_offset as u16;
				match value {
					DataVal::Integer(i) => sheet.write_number_with_format(chunk_row,col_offset,*i as f64, format)?,
					DataVal::Float(f) => sheet.write_number_with_format(chunk_row,col_offset, *f, format)?,
					DataVal::String(s) => sheet.write(chunk_row, col_offset, s)?,
				};//end matching type of data
			}//end looping over cells within row
			chunk_row += 1;
		}//end looping over the rows for this chunk

		// loop maintenance for writing multiple chunks
		chunk_row += 2;
	}//end writing each chunk of data to the sheet

	Ok(())
}