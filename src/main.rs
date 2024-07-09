use gui::GUI;
use rust_excel_helper::data::{self};

mod gui;

fn main() {
	let mut gui = GUI::initialize();
	let recv = gui.get_receiver();

	while gui.wait() {
		if let Some(msg) = recv.recv() {
			match msg {
				gui::InterfaceMessage::CSVInputFile(input_file) => {
					gui.start_wait();
					
					let data = data::read_csv_file(&input_file).unwrap();
					for dat in data {
						println!("\nFileID {}", dat.file_id);
						for line in dat.input_lines {
							println!("{:?}", line);
						}
					}

					gui.end_wait();
				},
				gui::InterfaceMessage::AppClosing => GUI::quit(),
				_ => println!("Message {:?} not recognized or supported.", msg),
			}//end matching based on the message
		}//end if we have an Interface Message
	}//end main app loop
}//end main method
