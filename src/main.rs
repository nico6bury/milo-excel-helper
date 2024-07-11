use gui::GUI;
use std::time::Instant;
use rust_excel_helper::{data::{self}, excel};

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
					let data = data::read_csv_file(&input_file).unwrap();

					// print out all data for debugging purposes
					for dat in data.iter() {
						println!("\nFileID {}", dat.file_id);
						println!("Ordering {:?}", dat.sample_ordering);
						for line in dat.input_lines.iter() {
							println!("{:?}", line);
						}//end looping over lines
					}//end looping over image file groups
					println!("\n\n\n");

					// figure out the output path we want
					let mut output_path = input_file.clone();
					output_path.set_file_name(format!("{}-OUT", input_file.file_name().unwrap().to_string_lossy()));
					output_path.set_extension("xlsx");

					// do a bunch of processing on data to get data chunks
					println!("Ready to start extracting data chunks from the data we read!");
					let labelled_chunks = excel::extract_labelled_chunks(&data);
					let sorted_1_chunks = excel::extract_sorted_chunks_1(&data);
					let sorted_2_chunks = excel::extract_sorted_chunks_2(&data);

					// write all the data chunks to various excel sheets
					let mut wb = excel::get_workbook();

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

					if let Ok(worksheet) = wb.worksheet_from_index(2) {worksheet.set_active(true);}

					println!("Closing the workbook, writing to {:?}", output_path);
					excel::close_workbook(&mut wb, &output_path).unwrap();
					println!("Finished writing to the workbook successfully!\n");

					gui.end_wait();
					let duration = start.elapsed();
					println!("Processing completed in {} milliseconds.", duration.as_millis());
				},
				gui::InterfaceMessage::AppClosing => GUI::quit(),
				_ => println!("Message {:?} not recognized or supported.", msg),
			}//end matching based on the message
		}//end if we have an Interface Message
	}//end main app loop
}//end main method
