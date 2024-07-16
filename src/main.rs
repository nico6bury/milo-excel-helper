use gui::GUI;
use std::time::{Duration, Instant};
use milo_excel_helper::{data::{self}, excel};

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
					let labelled_chunks = excel::extract_labelled_chunks(&data);
					let sorted_1_chunks = excel::extract_sorted_chunks_1(&data);
					let sorted_2_chunks = excel::extract_sorted_chunks_2(&data);
					let sum_chunk = excel::extract_sum_chunk(&data);
					let stats_chunk = excel::extract_stats_chunk(&data);
					let process_duration = process_start.elapsed();

					// write all the data chunks to various excel sheets
					let mut wb = excel::get_workbook();
					let workbook_start = Instant::now();

					println!("Writing data chunks to sheets!");
					excel::write_chunks_to_sheet(
						&mut wb,
						labelled_chunks.iter(),
						"labelled"
					).unwrap_or_else(|_|println!("Failed to write labelled chunks."));

					excel::write_chunks_to_sheet(
						&mut wb,
						sorted_1_chunks.iter(),
						"sorted-1"
					).unwrap_or_else(|_|println!("Failed to write sorted_1_chunks."));

					excel::write_chunks_to_sheet(
						&mut wb,
						sorted_2_chunks.iter(),
						"sorted-2"
					).unwrap_or_else(|_| println!("Failed to write sorted_2_chunks."));

					excel::write_chunks_to_sheet(
						&mut wb,
						vec![sum_chunk].iter(),
						"sum"
					).unwrap_or_else(|_| println!("Failed to write sum chunk."));

					excel::write_chunks_to_sheet(
						&mut wb,
						vec![stats_chunk].iter(),
						"stats"
					).unwrap_or_else(|_| println!("Failed to write stats chunk."));

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
