use gui::GUI;
use rust_xlsxwriter::{Workbook, XlsxError};
use std::time::{Duration, Instant};
use milo_excel_helper::{data::{self, InputFile}, excel::{self, DataChunk}};

mod gui;

fn main() {
	let mut gui = GUI::initialize();
	let recv = gui.get_receiver();

	while gui.wait() {
		if let Some(msg) = recv.recv() {
			match msg {
				gui::InterfaceMessage::CSVInputFile(input_file) => {
					gui.start_wait();
					let start = Instant::now();
					
					// get data from csv file
					let csv_start = Instant::now();
					let data = data::read_csv_file(&input_file).unwrap();
					let csv_duration = csv_start.elapsed();

					// print out all data for debugging purposes
					let debug_start = Instant::now();
					for dat in data.iter() {
						println!("\nFileID {}", dat.file_id);
						println!("Ordering {:?}", dat.sample_ordering);
						for line in dat.input_lines.iter() {
							println!("{:?}", line);
						}//end looping over lines
					}//end looping over image file groups
					println!("\n\n\n");
					let debug_duration = debug_start.elapsed();

					// do a bunch of processing on data to get data chunks
					println!("Ready to start extracting data chunks from the data we read!");
					let process_start = Instant::now();
					let detail_chunks = get_detail_chunks(&data);
					let sum_chunks = get_sum_chunks(&data);
					let process_duration = process_start.elapsed();

					// write all the data chunks to various excel sheets
					let mut wb = excel::get_workbook();
					let workbook_start = Instant::now();

					println!("Writing data chunks to sheets!");
					write_detail_chunks(&mut wb, detail_chunks)
						.unwrap_or_else(|_| println!("Failed to write some detailed chunks"));
					write_sum_chunks(&mut wb, sum_chunks)
						.unwrap_or_else(|_| println!("Failed to write some sum chunks"));
					if let Ok(worksheet) = wb.worksheet_from_index(4) {worksheet.set_active(true);}

					// figure out the output path we want for the xlsx file
					let mut output_path = input_file.clone();
					output_path.set_file_name(format!("{}-OUT", input_file.file_name().unwrap().to_string_lossy()));
					output_path.set_extension("xlsx");

					// actually write the changes to the workbook
					println!("Closing the workbook, writing to {:?}", output_path);
					excel::close_workbook(&mut wb, &output_path).unwrap();
					println!("Finished writing to the workbook successfully!\n");
					let workbook_duration = workbook_start.elapsed();
					
					gui.end_wait();

					// print information on how long program took to run
					let total_duration = start.elapsed();
					println!("{} milliseconds to read csv file.", format_milliseconds(csv_duration));
					println!("{} milliseconds to print debug information.", format_milliseconds(debug_duration));
					println!("{} milliseconds to process all data.", format_milliseconds(process_duration));
					println!("{} milliseconds to write all data to the workbook.", format_milliseconds(workbook_duration));
					println!("All processes completed after {} milliseconds.", format_milliseconds(total_duration));
				},
				gui::InterfaceMessage::AppClosing => GUI::quit(),
				_ => println!("Message {:?} not recognized or supported.", msg),
			}//end matching based on the message
		}//end if we have an Interface Message
	}//end main app loop
}//end main method

/// Given a duration, gives a string of a float representation of the number
/// of milliseconds. If the parse fails, it will return the whole
/// number of milliseconds as a string.
fn format_milliseconds(duration: Duration) -> String {
	match format!("{}",duration.as_micros()).parse::<f64>() {
		Err(_) => format!("{}",duration.as_millis()),
		Ok(micros) => format!("{0:.2}", micros / 1000.),
	}//end matching whether we can parse float-micros
}//end format_milliseconds(duration)

/// Shorthand for extracting:
/// - labelled_chunks
/// - sorted_1_chunks
/// - sorted_2_chunks
fn get_detail_chunks(
	data: &Vec<InputFile>
) -> (Vec<DataChunk>,Vec<DataChunk>,Vec<DataChunk>) {
	let labelled_chunks = excel::extract_labelled_chunks(&data);
	let sorted_1_chunks = excel::extract_sorted_chunks_1(&data);
	let sorted_2_chunks = excel::extract_sorted_chunks_2(&data);
	(labelled_chunks, sorted_1_chunks, sorted_2_chunks)
}//end get_detail_chunks

/// Shorthand for extracting:
/// - sum_chunk
/// - stats_chunk
fn get_sum_chunks(
	data: &Vec<InputFile>
) -> (DataChunk, DataChunk) {
	let sum_chunk = excel::extract_sum_chunk(data);
	let stats_chunk = excel::extract_stats_chunk(data);
	(sum_chunk, stats_chunk)
}//end get_sum_chunks

/// Shorthand for writing
/// - labelled chunks
/// - sorted-1 chunks
/// - sorted-2 chunks
fn write_detail_chunks(
	workbook: &mut Workbook,
	detail_chunks: (Vec<DataChunk>,Vec<DataChunk>,Vec<DataChunk>),
) -> Result<(), XlsxError> {
	let mut found_err = Ok(());
	excel::write_chunks_to_sheet(
		workbook,
		detail_chunks.0.iter(),
		"labelled"
	).unwrap_or_else(|err| {
		found_err = Err(err);
		println!("Failed to write labelled chunks.")
	});
	excel::write_chunks_to_sheet(
		workbook,
		detail_chunks.1.iter(),
		"sorted-1"
	).unwrap_or_else(|err| {
		found_err = Err(err);
		println!("Failed to write labelled chunks.")
	});
	excel::write_chunks_to_sheet(
		workbook,
		detail_chunks.2.iter(),
		"sorted-2"
	).unwrap_or_else(|err| {
		found_err = Err(err);
		println!("Failed to write labelled chunks.")
	});
	return found_err;
}//end write_detail_chunks()

/// Shorthand for writing
/// - sum chunk
/// - stats chunk
fn write_sum_chunks(
	workbook: &mut Workbook,
	sum_chunks: (DataChunk, DataChunk),
) -> Result<(),XlsxError> {
	let mut found_err = Ok(());
	excel::write_chunks_to_sheet(
		workbook,
		vec![sum_chunks.0].iter(),
		"sum"
	).unwrap_or_else(|err| {
		found_err = Err(err);
		println!("Failed to write labelled chunks.")
	});
	excel::write_chunks_to_sheet(
		workbook,
		vec![sum_chunks.1].iter(),
		"stats"
	).unwrap_or_else(|err| {
		found_err = Err(err);
		println!("Failed to write labelled chunks.")
	});
	return found_err;
}