use gui::GUI;
use rust_xlsxwriter::{Workbook, XlsxError};
use std::{path::PathBuf, time::{Duration, Instant}};
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
					process_and_time_files(&vec![input_file], false);
					gui.end_wait();
				},
				gui::InterfaceMessage::CSVInputFiles(files) => {
					gui.start_wait();
					process_and_time_files(&files, true);
					gui.end_wait();
				},
				gui::InterfaceMessage::AppClosing => GUI::quit(),
				_ => println!("Message {:?} not recognized or supported.", msg),
			}//end matching based on the message
		}//end if we have an Interface Message
	}//end main app loop
}//end main method

/// Does all the processing for a number of input files.  
/// Doesn't touch the gui, so you might want to do gui.start_wait()
/// and gui.end_wait() on your own.  
/// If output_sum_book is true, then a separate file will be created
/// with summary information across all files given.
fn process_and_time_files(files: &Vec<PathBuf>, output_sum_book: bool) {
	if files.len() == 0 {println!("Can't Batch Process 0 Files !!"); return;}
	let mut stats_chunks = Vec::new();
	println!("\n\n");
	let start = Instant::now();
	let mut csv_duration = Duration::new(0,0);
	let mut process_duration = Duration::new(0,0);
	let mut workbook_duration = Duration::new(0,0);
	
	for file in files.iter() {
		// get data from file
		let csv_instant = Instant::now();
		let data = data::read_csv_file(&file).expect("Failed to read csv input file!?");
		csv_duration += csv_instant.elapsed();

		// do processing to get data chunks
		let process_start = Instant::now();
		let detail_chunks = get_detail_chunks(&data);
		let sum_chunks = get_sum_chunks(&data);
		stats_chunks.push(sum_chunks.1.clone());
		process_duration += process_start.elapsed();

		// write all the data chunks to various excel sheets
		let workbook_start = Instant::now();
		let mut wb = excel::get_workbook();

		write_detail_chunks(&mut wb, detail_chunks)
			.unwrap_or_else(|_| println!("Failed writing detailed chunks for {}.", file.as_os_str().to_string_lossy()));
		write_sum_chunks(&mut wb, sum_chunks)
			.unwrap_or_else(|_| println!("Failed writing sum chunks for {}", file.as_os_str().to_string_lossy()));
		if let Ok(worksheet) = wb.worksheet_from_index(5) {worksheet.set_active(true);}

		// figure out output path we want for the xlsx file
		let mut output_path = file.clone();
		output_path.set_file_name(format!("{}-OUT", file.file_name().unwrap_or_default().to_string_lossy()));
		output_path.set_extension("xlsx");

		// actually write the changes to the workbook
		excel::close_workbook(&mut wb, &output_path)
			.unwrap_or_else(|_| println!("Failed to write changes to workbook!?"));
		workbook_duration += workbook_start.elapsed();

		println!("Finished all processes for file {}", file.file_name().unwrap_or_default().to_string_lossy());
	}//end doing all the processing for every file

	if output_sum_book {
		let mut wb = excel::get_workbook();
		let mut sum_book_output = files.first()
			.expect("We should have files at this point").clone();
		sum_book_output.set_file_name(format!("{}_file_summary_book", files.len()));
		sum_book_output.set_extension("xlsx");
		excel::write_chunks_to_sheet(
			&mut wb,
			stats_chunks.iter(),
			"all-stats"
		).unwrap_or_else(|_| println!("Failed to write stats to sum book."));
		excel::close_workbook(&mut wb, &sum_book_output)
			.unwrap_or_else(|_| println!("Failed to write changes to sum book."));
		println!("The summary sheet should be found at {}", sum_book_output.as_os_str().to_string_lossy());
	}//end if we're to output a summary book

	let total_duration = start.elapsed();
	println!("While processing {} files, took:", files.len());
	println!("- {} milliseconds to read csv files", format_milliseconds(csv_duration));
	println!("- {} milliseconds to process data", format_milliseconds(process_duration));
	println!("- {} milliseconds to write data to workbooks", format_milliseconds(workbook_duration));
	println!("And {} milliseconds for all processes and all files.", format_milliseconds(total_duration));
}//end process_and_time_files()

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
/// - sum chunk for area1
/// - sum chunk for area2
/// - sum_chunk for percent area
/// - stats chunk for area1
/// - stats_chunk for area2
/// - stats_chunk for percent area
fn get_sum_chunks(
	data: &Vec<InputFile>
) -> (DataChunk, DataChunk, DataChunk, DataChunk, DataChunk, DataChunk) {
	let sum_e_chunk = excel::extract_sum_chunk(data, excel::OutputVal::EndospermArea);
	let sum_k_chunk = excel::extract_sum_chunk(data, excel::OutputVal::KernelArea);
	let sum_p_chunk = excel::extract_sum_chunk(data, excel::OutputVal::PercentArea);
	let stats_e_chunk = excel::extract_stats_chunk(data, excel::OutputVal::EndospermArea);
	let stats_k_chunk = excel::extract_sum_chunk(data, excel::OutputVal::KernelArea);
	let stats_p_chunk = excel::extract_stats_chunk(data, excel::OutputVal::PercentArea);
	(sum_e_chunk, sum_k_chunk, sum_p_chunk, stats_k_chunk, stats_e_chunk, stats_p_chunk)
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
		println!("Failed to write sorted-1 chunks.")
	});
	excel::write_chunks_to_sheet(
		workbook,
		detail_chunks.2.iter(),
		"sorted-2"
	).unwrap_or_else(|err| {
		found_err = Err(err);
		println!("Failed to write sorted-2 chunks.")
	});
	return found_err;
}//end write_detail_chunks()

/// Shorthand for writing
/// - sum chunk
/// - stats chunk
fn write_sum_chunks(
	workbook: &mut Workbook,
	sum_chunks: (DataChunk, DataChunk, DataChunk, DataChunk, DataChunk, DataChunk),
) -> Result<(),XlsxError> {
	let mut found_err = Ok(());
	// write sum chunks
	excel::write_chunks_to_sheet(
		workbook,
		vec![sum_chunks.0, sum_chunks.1, sum_chunks.2].iter(),
		"sum"
	).unwrap_or_else(|err| {
		found_err = Err(err);
		println!("Failed to write sum chunks.")
	});
	// write stats chunks
	excel::write_chunks_to_sheet(
		workbook,
		vec![sum_chunks.3].iter(),
		"kernel-stats"
	).unwrap_or_else(|err| {
		found_err = Err(err);
		println!("Failed to write kernel Area stats chunks.")
	});
	excel::write_chunks_to_sheet(
		workbook,
		vec![sum_chunks.4].iter(),
		"ndsprm-stats"
	).unwrap_or_else(|err| {
		found_err = Err(err);
		println!("Failed to write endosperm Area stats chunks.")
	});
	excel::write_chunks_to_sheet(
		workbook,
		vec![sum_chunks.5].iter(),
		"%Area2-stats"
	).unwrap_or_else(|err| {
		found_err = Err(err);
		println!("Failed to write %Area2 stats chunks.")
	});
	return found_err;
}