use std::path::PathBuf;

use fltk::{app::{self, App, Receiver, Sender}, button::Button, dialog, enums::{Color, FrameType}, prelude::{ButtonExt, GroupExt, WidgetExt, WindowExt}, window::{self, Window}};

#[allow(dead_code)]
/// This enum is specifically intended for message passing
/// from the GUI to the main function. This is done
/// with Sender and Receiver objects created in initialize()
#[derive(Clone,PartialEq,Debug)]
pub enum InterfaceMessage {
    /// Indicates that the user has selected a CSV Input File.
    /// The filepath selected by the user is returned in the message.
    CSVInputFile(PathBuf),
    /// Indicates that the user has selected multiple CSV Input Files.
    /// THe filepaths selected by the user are returned in this message.
    CSVInputFiles(Vec<PathBuf>),
    /// Indicates that the user has clicked the Process Button,
    /// so they wish for the output file to be produced.
    ProcessSum,
    /// Indicates that the app is currently closing.
    AppClosing,
    /// Indicates that some other, unidentified message has been
    /// passed. In most cases, this is likely to be a mistake
    /// on the part of the sender.
    Other(String),
}//end enum InterfaceMessage

#[allow(dead_code)]
pub struct GUI {
	app: App,
	ux_main_window: Window,
	msg_sender: Sender<InterfaceMessage>,
	msg_receiver: Receiver<InterfaceMessage>
}//end struct GUI

#[allow(dead_code)]
impl GUI {
	/// Returns a clone of the receiver so you can
    /// react to messages sent by gui.
	pub fn get_receiver(&self) -> Receiver<InterfaceMessage> {
		return self.msg_receiver.clone();
	}//end get_receiver()

	/// Closes the application.
    pub fn quit() {
        app::App::default().quit();
    }//end show(self)

    /// Wraps app.wait().  
    /// To run main app loop, use while(gui.wait()){}.
    pub fn wait(&self) -> bool {
        self.app.wait()
    }//end wait(&self)

	/// Simply displays a message to the user.
    pub fn show_message(txt: &str) {
        dialog::message_default(txt);
    }//end show_message(txt)

    /// Simply displays an error message to the user.
    pub fn show_alert(txt: &str) {
        dialog::alert_default(txt);
    }//end show_alert(txt)

    /// Asks user a yes or no question. Returns true if
    /// user didn't close the dialog and clicked yes.
    pub fn show_yes_no_message(txt: &str) -> bool {
        match dialog::choice2_default(txt, "yes", "no", "") {
            Some(index) => index == 0,
            None => false,
        }//end matching dialog result
    }//end show_yes_no_message

    /// Asks the user to choose between three options.  
    /// If this is successful, returns index of choice, 0, 1, or 2
    pub fn show_three_choice(txt: &str, c0: &str, c1: &str, c2: &str) -> Option<u8> {
        match dialog::choice2_default(txt, c0, c1, c2) {
            Some(index) => {
                match u8::try_from(index) {
                    Ok(val) => Some(val),
                    Err(_) => None,
                }//end matching whether we can convert properly
            },
            None => None,
        }//end matching dialog result
    }//end show_three_choice()

	/// Gives a small visual indication that the program is doing something in the background.
    pub fn start_wait(&mut self) {
        self.ux_main_window.set_cursor(fltk::enums::Cursor::Wait);
    }//end start_wait(self)

    /// Clears the visual indication from start_wait()
    pub fn end_wait(&mut self) {
        self.ux_main_window.set_cursor(fltk::enums::Cursor::Default);
    }//end end_wait(self)

	/// Sets up all the properties and appearances of
	/// various widgets and UI setttings.
	pub fn initialize() -> GUI {
		let app = app::App::default();
		let mut main_window = window::Window::default()
			.with_size(200,75)
			.with_label("Milo");
		main_window.set_color(Color::from_rgb(255, 250, 240));
		main_window.end();

		let (s,r) = app::channel();

		// define some constants to be used repeatedly for sizing and styling
        let io_btn_width = 150;
        let io_btn_height = 30;
        let io_btn_frame = FrameType::GtkRoundUpFrame;
        let io_btn_down_frame = FrameType::GtkRoundDownFrame;
        let io_btn_color = Color::from_rgb(248,248,255);
        let io_btn_down_color = Color::from_rgb(240,255,240);

		// set up all the widgets and buttons for getting input and stuff
		let mut io_btn_get_input = Button::default()
			.with_pos(20,20)
			.with_size(io_btn_width, io_btn_height)
			.with_label("Select Input CSV(s)");
		io_btn_get_input.set_frame(io_btn_frame);
		io_btn_get_input.set_down_frame(io_btn_down_frame);
		io_btn_get_input.set_color(io_btn_color);
		io_btn_get_input.set_selection_color(io_btn_down_color);
		main_window.add(&io_btn_get_input);

		io_btn_get_input.set_callback({
			let sender_clone = s.clone();
			move |_| {
				if let Err(err_message) = GUI::create_io_dialog(
                    &sender_clone,
                    dialog::NativeFileChooserType::BrowseMultiFile,
                    dialog::NativeFileChooserOptions::UseFilterExt,
                    "*.csv",
                    "Please select a csv input file"
                ) {
					println!("Encountered an error when attempting to show file dialog:\n{}", err_message);
				}//end if we got an error
			}//end moving for closure
		});

		// set some final options for main window
		main_window.make_resizable(true);
		main_window.set_callback({
			let sender_clone = s.clone();
			move |_| {
				sender_clone.send(InterfaceMessage::AppClosing);
				println!("World is Ending!");
			}
		});
		main_window.show();

		GUI {
			app,
			ux_main_window: main_window,
			msg_sender: s,
			msg_receiver: r,
		}//end struct construction
	}//end initialize()

	/// Helper method used in initialize to share code between handlers
    /// of io buttons.
    fn create_io_dialog(sender: &Sender<InterfaceMessage>, dialog_type: dialog::NativeFileChooserType, dialog_option: dialog::NativeFileChooserOptions, dialog_filter: &str, dialog_title: &str ) -> Result<(), String> {
        // set up dialog with all the settings
        let mut dialog = dialog::NativeFileChooser::new(dialog_type);
        dialog.set_option(dialog_option);
        dialog.set_filter(dialog_filter);
        dialog.set_title(dialog_title);
        dialog.show();
        // make sure the dialog didn't have an error
        let dialog_error = dialog.error_message().unwrap_or_else(|| "".to_string()).replace("No error", "");
        if dialog_error != "" {
            return Err(format!("We encountered a dialog error somehow. Details below:\n{}", dialog_error));
        }//end if dialog had an error
        let dialog_filenames = dialog.filenames();
        drop(dialog);
        // make sure we can get the file from the dialog
        match dialog_filenames.len() {
            n if n <= 0 => sender.send(InterfaceMessage::Other("No files selected for input!?".to_string())),
            1 => sender.send(InterfaceMessage::CSVInputFile(dialog_filenames.first().expect("checked length for .first()").clone())),
            _ => sender.send(InterfaceMessage::CSVInputFiles(dialog_filenames))
        }//end matching right message to length
        return Ok(());
    }//end create_io_dialog()
}//end impl for GUI